<script lang="ts">
  import { onMount } from "svelte";
  import {
    Alert,
    Badge,
    Button,
    Checkbox,
    Label,
    ListPlaceholder,
    Listgroup,
    Modal,
    Search,
    Textarea,
  } from "flowbite-svelte";
  import { CloseOutline } from "flowbite-svelte-icons";
  import NewClassModal from "$lib/components/shell/NewClassModal.svelte";
  import type { NewClassInput } from "$lib/components/shell/NewClassModal.svelte";
  import { Table as DataTable } from "@flowbite-svelte-plugins/datatable";
  /*
    Tipler derin yoldan — paket girişi yalnız DEĞERLERİ dışa veriyor, tipler
    `dist/dts/types`ta. Gerekçenin tamamı `routes/exams/+page.svelte`te.
  */
  import type {
    DataTableOptions,
    elementNodeType,
    nodeType,
  } from "simple-datatables/dist/dts/types";
  import PageShell from "$lib/components/shell/PageShell.svelte";
  import { pushEscapeLayer } from "$lib/ui/escape-stack";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import type { Classroom, Student } from "$lib/types";

  let classrooms = $state<Classroom[]>([]);

  /**
   * Seçili sınıflar — sayfanın tek gezinme durumu.
   *
   * SEÇİM TEK YERDE: sol sütun. Bir ara ortada da bir çoklu seçim açılırı
   * vardı ve ikisi aynı diziyi yazıyordu; aynı ekranda aynı işi yapan iki
   * denetim, hangisinin geçerli olduğunu belirsizleştirmekten başka bir şey
   * yapmıyordu. Arama da soldan alınıp listenin başına konuldu — arama
   * ÖĞRENCİ listesinin işi, sınıf seçiminin değil.
   *
   * `students` BAĞIMSIZ DURUM DEĞİL, `allStudents`tan türüyor: sınıf seçmek
   * tek bir API çağrısı bile yapmıyor, dolayısıyla bekleme durumu da yok.
   */
  let selectedIds = $state<string[]>([]);
  /** Yalnız İLK yükleme. Sonraki tazelemeler iskeleti tekrar göstermiyor. */
  let loading = $state(true);
  let error = $state<string | null>(null);
  let busy = $state(false);

  // Yeni sınıf — form kalıbın içinde, durumu da orada.
  let newClassOpen = $state(false);
  let newClassError = $state<string | null>(null);

  // Toplu öğrenci girişi — satır başına "numara,ad soyad"
  let bulkText = $state("");

  /** Öğrenci listesindeki arama — yalnız SEÇİLİ sınıfların satırlarını süzer. */
  let studentQuery = $state("");

  /**
   * Bütün sınıfların öğrencileri — tablonun tek kaynağı.
   *
   * Sınıf başına ayrı çağrı yerine hepsi bir kez yükleniyor ki `students` saf
   * bir `$derived` olabilsin: seçim değişince ne IPC gidiyor ne de sıralamayı
   * bozacak bir yarış oluşuyor. Yerel SQLite ve bir öğretmenin sınıf sayısı
   * küçük; maliyet açılışta birkaç milisaniye.
   */
  let allStudents = $state<Student[]>([]);

  let selectedSet = $derived(new Set(selectedIds));

  let students = $derived(allStudents.filter((student) => selectedSet.has(student.classroom_id)));

  /**
   * Türkçe küçültme. `toLowerCase()` "İSTANBUL"u bozar; Türkçede I → ı,
   * İ → i. Öğrenci adları da sınıf adları da Türkçe.
   */
  function lower(text: string): string {
    return text.toLocaleLowerCase("tr");
  }

  /**
   * Sınıf başına öğrenci sayısı — GERÇEK satırlardan.
   *
   * Rozet eskiden `classroom.student_ids.length` okuyordu. O alan üyeliğin
   * ikinci bir kopyasıydı ve hiç bakımı yapılmıyordu: yalnız `seed_demo`
   * dolduruyor, `add_student` komutu dokunmuyor, çıkarma yolu hiç yok.
   * Arayüzden açılan bir sınıf kaç öğrenci alırsa alsın rozette 0 kalırdı;
   * tohumlanmış bir sınıfta da silinen öğrenciden sonra sayı düşmezdi. Alan
   * tümüyle kaldırıldı, sayı `allStudents`tan türüyor — tablo neyi
   * gösteriyorsa rozet onu sayıyor.
   */
  let studentCountByClass = $derived(
    Object.fromEntries(
      classrooms.map((c) => [c.id, allStudents.filter((s) => s.classroom_id === c.id).length]),
    ) as Record<string, number>,
  );

  let classNameById = $derived(
    Object.fromEntries(classrooms.map((c) => [c.id, c.name])) as Record<string, string>,
  );

  let studentNeedle = $derived(lower(studentQuery.trim()));

  /**
   * Süzgeç TEK BİR birleşik metinde arıyor, üç alanda ayrı ayrı değil.
   *
   * Eskiden `number`, `first_name` ve `last_name` üç ayrı `includes`ti:
   * tabloda "Ayşe Yılmaz" yazdığı hâliyle aranınca HİÇBİR ŞEY eşleşmiyordu,
   * çünkü aradaki boşluğu içeren bir alan yok. Alanları birleştirmek hem tam
   * adı hem çok kelimeli ön adları kendiliğinden çözüyor.
   *
   * SINIF ADI BİLEREK DIŞARIDA. Sınıf adları rakamla başlıyor ("9-A") ve
   * öğrenci numaraları da rakam: "9" yazmak 9-A'nın tüm listesini ve içinde 9
   * geçen her numarayı getirirdi. Sınıfa göre gruplamak için tablonun "Sınıf"
   * sütun başlığına tıklamak yeterli.
   */
  let filteredStudents = $derived(
    studentNeedle === ""
      ? students
      : students.filter((student) =>
          lower(`${student.number} ${student.first_name} ${student.last_name}`).includes(
            studentNeedle,
          ),
        ),
  );

  onMount(loadClassrooms);

  async function loadClassrooms() {
    /*
      `error = null` AWAİTLERDEN ÖNCE. Eskiden en sonda duruyordu ve araya
      giren bir çağrının kendi içinde yazdığı hatayı siliyordu: yazma başarılı,
      ardındaki yeniden yükleme başarısız olduğunda ekran "her şey yolunda"
      diyordu. Aynı desen üç yerde vardı.
    */
    error = null;
    try {
      classrooms = await api.students.listClassrooms();
      if (selectedIds.length === 0 && classrooms.length > 0) selectedIds = [classrooms[0].id];
      await loadAllStudents();
    } catch (err: unknown) {
      error = errorText(err);
    } finally {
      loading = false;
    }
  }

  /**
   * Bütün sınıfların öğrencilerini toplar — tablonun kaynağı.
   *
   * SUNUCUDA ARAMA YOK: `api.students` yalnız sınıfa göre listeleme veriyor.
   * Çağrılar paralel gidiyor.
   *
   * HATA ANA ALANA YAZILIYOR: sessizce başarısız olursa öğretmen boş bir tablo
   * görür ve sebebini bilmez.
   */
  async function loadAllStudents() {
    try {
      const lists = await Promise.all(
        classrooms.map((c) => api.students.listByClassroom(c.id)),
      );
      allStudents = lists.flat();
    } catch (err: unknown) {
      error = errorText(err);
      throw err;
    }
  }

  /**
   * TOPLU SEÇ / TEMİZLE ŞART, süs değil.
   *
   * Onay kutuları tek tek açılıp kapanıyor; "Temizle" olmadan 12 sınıftan
   * 1'ine inmek 11 tıklama eder. Toplu ekleme paneli TEK sınıf seçiliyken
   * çalıştığı için o panel pratikte hiç açılmazdı. İkisi de klavyeyle
   * erişilebilir düğme — fareye özel bir kısayolun arkasında değil.
   */
  function selectAllClasses() {
    selectedIds = classrooms.map((c) => c.id);
  }

  function clearClasses() {
    selectedIds = [];
  }

  /**
   * Kalıptan gelen sınıf oluşturma — elle ya da Excel'den.
   *
   * KISMİ YAZMA GERÇEK BİR RİSK. Bu yığında toplu ekleme komutu da işlem
   * (transaction) da YOK: sınıf bir çağrıyla, her öğrenci ayrı bir çağrıyla
   * yazılıyor. 30 öğrencinin 17'sinde hata alırsak sınıf ve 16 öğrenci
   * VERİTABANINDA KALIYOR — geri alınamıyor. Bu yüzden:
   *   • Hata mesajı kaç öğrencinin yazıldığını AÇIKÇA söylüyor.
   *   • Hata yolunda da liste yeniden yükleniyor; yazılanlar görünmez kalırsa
   *     öğretmen aynı dosyayı tekrar yükler ve hepsini ÇİFTLER (şemada
   *     (classroom_id, number) üzerinde UNIQUE kısıt yok).
   *
   * `false` dönerse kalıp açık kalıyor ve hata orada görünüyor.
   */
  async function createClass(input: NewClassInput): Promise<boolean> {
    busy = true;
    newClassError = null;
    let written = 0;
    let classId: string | null = null;

    try {
      classId = await api.students.createClassroom({
        name: input.name,
        grade: input.grade,
        branch: input.branch,
      });

      for (const student of input.students) {
        await api.students.addStudent({
          classroom_id: classId,
          number: student.number,
          first_name: student.first,
          last_name: student.last,
        });
        written++;
      }

      await loadClassrooms();
      selectedIds = [classId];
      dataVersion++;
      return true;
    } catch (err: unknown) {
      const detail = errorText(err);
      newClassError =
        classId === null
          ? `Sınıf oluşturulamadı: ${detail}`
          : `Sınıf oluşturuldu ve ${written} öğrenci eklendi, sonrası başarısız: ${detail}. ` +
            `Aynı dosyayı tekrar yüklersen bu ${written} öğrenci ÇİFTLENİR — önce listeyi kontrol et.`;

      // Yazılanlar görünür olsun; sessizce kalmaları çiftlemeye davetiye.
      try {
        await loadClassrooms();
        if (classId) selectedIds = [classId];
        dataVersion++;
      } catch {
        // `loadClassrooms` hatayı zaten `error`a yazdı; asıl mesajı ezmiyoruz.
      }
      return false;
    } finally {
      busy = false;
    }
  }

  /**
   * Toplu giriş: satır başına "numara,ad soyad". Öğretmen listeyi e-okuldan
   * kopyalayıp yapıştırır; tek tek form doldurmak gerçek bir zaman kaybı.
   */
  type ParsedStudent = { number: string; first: string; last: string };

  function parseBulk(text: string): ParsedStudent[] {
    return text
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean)
      .map((line) => {
        const [numberPart, ...nameParts] = line.split(/[,;\t]/);
        const fullName = nameParts.join(" ").trim();
        const words = fullName.split(/\s+/).filter(Boolean);
        const last = words.length > 1 ? (words.pop() as string) : "";
        return { number: numberPart.trim(), first: words.join(" "), last };
      })
      .filter((s) => s.number !== "" && s.first !== "");
  }

  let parsed = $derived(parseBulk(bulkText));

  /**
   * Toplu eklemenin hedefi.
   *
   * TEK SINIF ŞART: "bu satırları hangi sınıfa yazayım?" sorusunun çoklu
   * seçimde tek cevabı yok. Tam bir sınıf seçiliyse target odur; değilse
   * ekleme kapalı ve sebebi yazıyor.
   */
  let bulkTarget = $derived(selectedIds.length === 1 ? selectedIds[0] : null);

  // ————— Öğrenci tablosu —————

  /**
   * Satırlar. Anahtar sırası sütun sırası, anahtar metni sütun başlığı —
   * eklenti başlıkları `Object.keys(items[0])` ile üretiyor.
   *
   * `İşlem` sütununun DEĞERİ öğrencinin KİMLİĞİ. Görünürde bir "Sil" düğmesi
   * çiziliyor ama altındaki hücre verisi id; silme işlemi böylece satır
   * indeksine hiç bağlanmıyor. Bu bilinçli: `simple-datatables` sıralamayı
   * `data.data` dizisini yeniden kurarak yapıyor, yani `render`a gelen indeks
   * kaynak dizideki konumla eşleşmiyor — Sınavlar sayfasında tam bu tuzak
   * yanlış sınavın açılmasına yol açmıştı.
   */
  let studentRows = $derived(
    filteredStudents.map((student) => ({
      No: student.number,
      "Ad Soyad": `${student.first_name} ${student.last_name}`.trim(),
      /*
        SINIF SÜTUNU: birden çok sınıf birlikte gösterilebildiği için satırın
        hangi sınıfa ait olduğu artık listeden okunamıyor.
      */
      Sınıf: classNameById[student.classroom_id] ?? "",
      İşlem: student.id,
    })),
  );

  /**
   * Veri sürümü — her başarılı yeniden yükleme sonrası artıyor.
   *
   * SAYAÇ, UZUNLUK DEĞİL. Aşağıdaki anahtarda `students.length` yazsaydık aynı
   * sayıda ama farklı satır dönen bir tazeleme — `createClass`ın hata yolundaki
   * yeniden yükleme gibi — anahtarı değiştirmez ve ekranda ESKİ satırlar
   * kalırdı.
   */
  let dataVersion = $state(0);

  /** Süzgeç değişince tablo yeniden kurulmalı: eklenti satırları yalnız kurulurken okuyor. */
  let tableKey = $derived(`${selectedIds.join(",")}|${studentNeedle}|${dataVersion}`);

  function textNode(value: string): nodeType {
    return { nodeName: "#text", data: value };
  }

  /**
   * Hücrenin içini değiştirip hücrenin KENDİSİNİ döndürür.
   *
   * `render` bir nesne döndürünce `simple-datatables` onu `<td>`NİN YERİNE
   * koyuyor; doğrudan bir `<button>` döndürmek `<td>`yi yok eder ve `<tr>`
   * içinde geçersiz düğüm bırakırdı. String de döndürülmüyor: string dönüşü
   * `<td>${i}</td>` olarak AYRIŞTIRILIYOR ve öğrenci adı ham kullanıcı metni.
   */
  function fillCell(td: object, children: nodeType[]): elementNodeType {
    const cell = td as elementNodeType;
    cell.childNodes = children;
    return cell;
  }

  /*
    Kenarlık elle yazılı: `border-gray-300 dark:border-gray-600` çiftinin tam
    karşılığı olan bir Flowbite değişkeni yok. En yakını `default-strong`
    (gray-200/gray-600) ve açık kipte kenarlığı zayıflatıyor.
  */
  const DELETE_BUTTON =
    "rounded-lg border border-gray-300 px-2 py-1 text-xs text-heading " +
    "hover:bg-neutral-tertiary-medium dark:border-gray-600";

  const studentTableOptions: DataTableOptions = {
    /* Arama Flowbite'ın `<Search>` bileşeninde; eklentininki kapalı. */
    searchable: false,
    sortable: true,
    paging: true,
    perPage: 25,
    perPageSelect: [25, 50, 100],
    /* Sıralama Türkçe harmanlamayla: "Çevik" C'lerin önüne düşmesin. */
    locale: "tr",
    labels: {
      placeholder: "",
      searchTitle: "",
      searchLabel: "",
      perPage: "kayıt/sayfa",
      pageTitle: "Sayfa {page}",
      noRows: "Öğrenci yok",
      noResults: "Eşleşen öğrenci yok",
      info: "{rows} öğrencinin {start}–{end} arası",
      sortHint: "Sıralamak için etkinleştir",
    },
    columns: [
      { select: 0, type: "number", cellClass: "tnum text-body-subtle" },
      { select: 1, type: "string" },
      { select: 2, type: "string", cellClass: "text-body-subtle" },
      {
        /* Sınıf sütunu eklenince 2'den 3'e kaydı — kaçırılırsa `render` yanlış
           hücreyi süsler ve ok gizleme de yanlış başlığa gider. */
        select: 3,
        /* Sıralanacak bir şey değil — hücrenin verisi id, görünen ise düğme. */
        sortable: false,
        /*
          BAŞLIKTAKİ SIRALAMA OKUNU GİZLE. Eklentinin `TableHeadCell`i ok
          ikonunu sütun bazında değil, tablo genelindeki `sortable` bağlamına
          bakarak çiziyor; ikon başlığın İÇERİĞİNE giriyor ve
          `simple-datatables` onu olduğu gibi taşıyor. Sonuç: sıralanmayan bir
          sütunda sıralanabilir görünen bir ok — tıklayınca hiçbir şey olmuyor.
          `headerClass` bir prop, elle yazılmış CSS katmanı değil.
        */
        headerClass: "[&_svg]:hidden",
        type: "string",
        render: (value, td) =>
          fillCell(td, [
            {
              nodeName: "BUTTON",
              attributes: {
                type: "button",
                class: DELETE_BUTTON,
                "data-delete-student": String(value),
              },
              childNodes: [textNode("Sil")],
            },
          ]),
      },
    ],
  };

  let tableHost = $state<HTMLElement | null>(null);

  /**
   * Silinmek üzere seçilen öğrenci — onay kalıbının konusu.
   *
   * SİLME ONAYSIZ OLAMAZ. `delete_student` geri alınamıyor, geri alma yok ve
   * düğme tablonun her satırında duruyor: tek bir kaçık tıklama öğrenciyi
   * kaydıyla birlikte götürür.
   */
  let pendingDelete = $state<Student | null>(null);

  /**
   * Silme tıklamalarını yakalar.
   *
   * DİNLEYİCİ EMİR KİPİYLE BAĞLANIYOR, şablonda `onclick` ile değil: düğmeleri
   * Svelte değil `simple-datatables` çiziyor, dolayısıyla şablonda onlara
   * erişim yok. Kaba `onclick` yazmak da Svelte'in haklı a11y uyarısını
   * ("rolü olmayan `<div>` tıklama dinleyemez") getirirdi.
   *
   * `dataset.deleteStudent` — `data-delete-student` özniteliğinin camelCase
   * karşılığı. Bu köprü tip denetiminin DIŞINDA: `DOMStringMap` bir dizin
   * imzası, her anahtarı `string | undefined`, dolayısıyla yanlış yazılmış bir
   * anahtar hata vermez, sessizce `undefined` döner ve silme hiç çalışmaz.
   */
  $effect(() => {
    const host = tableHost;
    if (!host) return;

    const onDeleteClick = (event: Event) => {
      const target = event.target as HTMLElement | null;
      const button = target?.closest<HTMLElement>("[data-delete-student]");
      const id = button?.dataset.deleteStudent;
      if (!id || busy) return;

      pendingDelete = allStudents.find((student) => student.id === id) ?? null;
    };

    host.addEventListener("click", onDeleteClick);
    return () => host.removeEventListener("click", onDeleteClick);
  });

  /**
   * Onay kalıbı Esc merdivenine kayıtlı.
   *
   * Flowbite'ın `Modal`ı `<dialog>` + `showModal()` üstüne kurulu, yani
   * tarayıcının kendi Esc'i var; ama uygulamanın tek pencere dinleyicili Esc
   * merdiveni `preventDefault()` çağırıyor. Kayıt olmazsa Esc bu kalıbı değil
   * ALTTAKİ katmanı kapatabilirdi.
   */
  $effect(() => {
    if (!pendingDelete) return;
    return pushEscapeLayer(() => (pendingDelete = null));
  });

  async function confirmDelete() {
    const student = pendingDelete;
    if (!student || busy) return;

    busy = true;
    error = null;
    try {
      await api.students.deleteStudent(student.id);
      pendingDelete = null;
      /*
        `loadClassrooms`, `loadAllStudents` değil: sol sütundaki sayaç rozeti
        `allStudents`tan türüyor ve sınıf listesi de tazelenmeli; yalnız
        birini yenilemek ikisini ayrı düşürürdü.
      */
      await loadClassrooms();
      dataVersion++;
    } catch (err: unknown) {
      error = errorText(err);
    } finally {
      busy = false;
    }
  }

  async function addBulk() {
    if (!bulkTarget || parsed.length === 0) return;
    busy = true;
    error = null;
    try {
      for (const s of parsed) {
        await api.students.addStudent({
          classroom_id: bulkTarget,
          number: s.number,
          first_name: s.first,
          last_name: s.last,
        });
      }
      bulkText = "";
      await loadClassrooms();
      dataVersion++;
    } catch (err: unknown) {
      error = errorText(err);
    } finally {
      busy = false;
    }
  }

  /**
   * Bölüm başlığı — üç sütunda da aynı.
   *
   * Flowbite'ın `Heading`i burada işe yaramıyor: teması `text-4xl font-bold`
   * ve bu ölçüye indirmek katkısının neredeyse tamamını geri almayı gerektirir
   * — yani elle yazılan sınıfları AZALTMAZ, artırır. Küçük harfli bir bölüm
   * etiketi ya da yapışkan başlık bileşeni pakette yok. Renkler yine de
   * Flowbite değişkenleri: `text-body-subtle`, `bg-neutral-secondary-soft` ve
   * `border-default-medium` kendi içlerinde koyu kip karşılıklarını taşıyor
   * (flowbite/src/themes/default.css bunları `.dark` altında yeniden
   * tanımlıyor), bu yüzden tek bir `dark:` yok.
   */
  const SECTION_HEADING =
    "shrink-0 border-b border-default-medium bg-neutral-secondary-soft px-4 py-2 " +
    "text-xs font-semibold uppercase tracking-wide text-body-subtle";
</script>

<!--
  scroll={false}: üç sütun (sınıflar / öğrenciler / toplu ekle) her biri kendi
  kaydırıcısını taşıyor. Kabuk da kaydırırsa fare tekerleğinin hangi sütunu
  süreceği belirsizleşir — PageShell'in kendi uyarısı da bunu söylüyor.
-->
<PageShell
  title="Öğrenciler"
  subtitle={loading
    ? null
    : `${selectedIds.length}/${classrooms.length} sınıf · ${filteredStudents.length} öğrenci`}
  scroll={false}
>
  <div class="flex h-full min-h-0 flex-col">
    {#if error}
      <Alert color="red" class="m-4 shrink-0">{error}</Alert>
    {/if}

    <!--
      Yan izler sabit (240px / 300px), orta iz minmax(0,1fr).
      Bir ara yan izlere minmax(180px,240px) ve minmax(240px,300px) verilmişti;
      ölçüldü: desteklenen HİÇBİR pencere boyutunda sonucu değiştirmiyordu, o
      yüzden geri alındı. Grid iz boyutlandırmasında (§12.6 Maximize Tracks)
      esnek olmayan izler önce büyüme sınırlarına kadar doldurulur, fr iz ancak
      artan yeri alır. En dar desteklenen pencerede (tauri.conf.json
      minWidth=1024) alan 1024 − 224 (w-56 çekmece) − 1 (border-r) = 799px:
      tabanlar 180 + 0 + 240 = 420, boş yer 379 ve yan izlerin sınırlarına
      çıkması için gereken 60 + 60 = 120'den büyük ⇒ T1 240'a, T3 300'e
      doluyor, ortaya 799 − 540 = 259px kalıyor. Sabit izlerle bire bir aynı
      sonuç; alt sınırlar ancak alan 540px'in altına inseydi iş görürdü ve
      minWidth=1024 oraya inilmesine izin vermiyor.
      Yani orta sütunu 259px'e sığdıran şey ızgara değil, aşağıdaki hücre
      düzeltmeleri: px-3, w-px ve wrap-anywhere.
      minmax(0,1fr) korunuyor: çıplak `1fr` aslında minmax(auto,1fr) demek,
      o zaman orta izin tabanı section'daki overflow-auto'nun otomatik en küçük
      boyu 0'a düşürmesine bağlı kalırdı. 0 tabanı burada açıkça yazılı.

      Grid'in HÜCRELERİ artık bütünüyle `overflow-auto` değil, `flex flex-col`:
      her sütunun başlığı ve eylem şeridi sabit kalıp yalnız ORTA parça kayıyor.
      Eskiden sütunun tamamı kayıyordu ve sol sütunun "Yeni sınıf" düğmesi uzun
      sınıf listesinde ekranın altına düşüp erişilemez oluyordu.
    -->
    <div class="grid min-h-0 flex-1 grid-cols-[240px_minmax(0,1fr)_300px]">
      <!-- ————— Sol: SINIF SEÇİMİ ————— -->
      <section class="flex min-h-0 flex-col border-r border-default-medium">
        <div class="{SECTION_HEADING} flex items-center justify-between gap-2">
          <h2 id="classroom-list-heading">Sınıflar</h2>
          <span class="font-normal normal-case tracking-normal">
            {selectedIds.length}/{classrooms.length}
          </span>
        </div>

        <div class="min-h-0 flex-1 overflow-auto p-3">
          {#if loading}
            <!--
              İSKELET, "Sınıf yok." DEĞİL. İlk yükleme sürerken liste boş
              olduğu için ekran öğretmene hiç sınıfı olmadığını söylüyordu —
              oysa veri daha geliyordu. `ListPlaceholder` kendi `role="status"`
              unu taşıyor.
            -->
            <ListPlaceholder itemNumber={5} />
          {:else if classrooms.length === 0}
            <p class="text-sm text-body-subtle">Sınıf yok.</p>
          {:else}
            <!--
              ONAY KUTUSU, `aria-pressed`Lİ DÜĞME DEĞİL.

              Çoklu seçimin doğru anlatımı `<input type="checkbox">`: işaret,
              Boşluk tuşuyla açıp kapama, Sekme ile erişim ve etiketin
              erişilebilir ad olması platformdan geliyor. Eskiden bunun yerine
              `aria-pressed` + üç ayrı `class:` anahtarı vardı ve satır tek
              seçimliymiş gibi görünüyordu.

              `Listgroup` `active` ALMIYOR — kaynakta `tag = active ? "div" :
              "ul"`, yani `active` verseydik gerçek bir liste olmaktan çıkıp
              `<div>` olurdu ve ekran okuyucu "liste, N öğe" demezdi.
              `ListgroupItem`in `current`i de kullanılmıyor: teması koyu kipte
              `dark:bg-gray-800`, kabın kendi zemini de `dark:bg-gray-800` —
              seçim görünmez olurdu.

              `divClass`, `classes` DEĞİL: `classes` bileşende okunuyor ama
              `CheckboxProps` tipinde YOK, yazınca `svelte-check` hata verir.
              `divClass` tipli; karşılığında geliştirme kipinde bir kullanımdan
              kaldırma uyarısı düşüyor, üretimde sessiz.
            -->
            <Listgroup aria-labelledby="classroom-list-heading">
              {#each classrooms as classroom (classroom.id)}
                <li class="has-[:checked]:bg-neutral-tertiary-medium">
                  <Checkbox
                    bind:group={selectedIds}
                    value={classroom.id}
                    divClass="w-full cursor-pointer gap-2 px-4 py-2"
                  >
                    <span class="min-w-0 flex-1 truncate">{classroom.name}</span>
                    <Badge color="primary" rounded>{studentCountByClass[classroom.id] ?? 0}</Badge>
                  </Checkbox>
                </li>
              {/each}
            </Listgroup>
          {/if}
        </div>

        <div class="shrink-0 space-y-2 border-t border-default-medium p-3">
          <div class="flex gap-1">
            <Button
              size="xs"
              color="alternative"
              class="flex-1"
              disabled={classrooms.length === 0}
              onclick={selectAllClasses}
            >
              Tümünü seç
            </Button>
            <Button
              size="xs"
              color="alternative"
              class="flex-1"
              disabled={selectedIds.length === 0}
              onclick={clearClasses}
            >
              Temizle
            </Button>
          </div>
          <!--
            FORM KALIBA TAŞINDI. Sol sütunda dört alan + düğme, sınıf listesini
            aşağı itip görünmez hâle getiriyordu; üstelik Excel'den içe aktarma
            eklenince oraya sığacak yer hiç kalmıyordu.
          -->
          <Button size="sm" class="w-full" onclick={() => (newClassOpen = true)}>
            Yeni sınıf
          </Button>
        </div>
      </section>

      <!-- ————— Orta: ÖĞRENCİ LİSTESİ + ARAMA ————— -->
      <section class="flex min-h-0 flex-col border-r border-default-medium">
        <h2 class={SECTION_HEADING}>Öğrenci listesi</h2>

        <div class="min-h-0 flex-1 overflow-auto">
          <!--
            ARAMA LİSTENİN BAŞINDA, sol sütunda değil: aradığı şey ÖĞRENCİ ve
            kapsamı soldan seçilmiş sınıflar. Bir ara solda "tüm sınıflarda
            ara" diye ikinci bir kutu vardı; aynı ekranda iki arama kutusu,
            hangisinin ne aradığını belirsizleştirmekten başka iş görmüyordu.

            `max-w-72`, `w-72` DEĞİL. 18rem = 288px, ama en dar desteklenen
            pencerede orta izin genişliği yukarıdaki hesaba göre 259px:
            `w-72` sütunu 29px taşırıp yatay kaydırıcı doğuruyordu.

            `clearable` BİLEREK YOK. Kaynakta `clearAll` bağlı değeri `""`
            değil `undefined` yapıyor; `studentQuery.trim()` o anda
            `TypeError: Cannot read properties of undefined (reading 'trim')`
            fırlatır ve türetilmiş zinciri götürür. Temanın kapatma ikonundaki
            `hover:text-black`ın koyu kip karşılığı da yok.

            `required={false}` YÜK TAŞIYOR: şablonda `required` sabit yazılı,
            yalnız `{...restProps}` sonradan yayıldığı için geri alınabiliyor.
            Kaldırılırsa kutu hiçbir formun içinde olmadığı hâlde sürekli
            `:invalid` görünür.
          -->
          <div class="max-w-72 px-4 py-3">
            <Label for="student-search" class="mb-1.5">Seçili sınıflarda ara</Label>
            <Search
              id="student-search"
              size="md"
              required={false}
              disabled={selectedIds.length === 0}
              bind:value={studentQuery}
              placeholder="Numara veya ad…"
            />
          </div>

          {#if selectedIds.length === 0}
            <p class="px-4 pb-4 text-sm text-body-subtle">Soldan en az bir sınıf seç.</p>
          {:else if students.length === 0}
            <p class="px-4 pb-4 text-sm text-body-subtle">Seçili sınıflarda öğrenci yok.</p>
          {:else if filteredStudents.length === 0}
            <!--
              Eklenti boş `items` ile hiç `<thead>`/`<tbody>` çizmiyor; kendi
              "sonuç yok" mesajı da o durumda görünmüyor. Mesaj bu yüzden burada.
            -->
            <p class="px-4 pb-4 text-sm text-body-subtle">Aramanla eşleşen öğrenci yok.</p>
          {:else}
            <!--
              `bind:this`: silme düğmelerini Svelte değil `simple-datatables`
              çiziyor, tıklamaları bu kapta emir kipiyle dinliyoruz. Kap
              `{#key}`in DIŞINDA kalmalı, yoksa her yeniden kurulumda dinleyici
              düşer.
            -->
            <div class="px-4 pb-4" bind:this={tableHost}>
              {#key tableKey}
                <DataTable items={studentRows} dataTableOptions={studentTableOptions} />
              {/key}
            </div>
          {/if}
        </div>
      </section>

      <!-- ————— Sağ: TOPLU EKLE ————— -->
      <aside class="flex min-h-0 flex-col">
        <h2 class={SECTION_HEADING}>Toplu ekle</h2>

        <div class="min-h-0 flex-1 overflow-auto px-4 py-3">
          <p class="text-sm text-body-subtle">
            Satır başına bir öğrenci: numara, ad soyad. Listeyi olduğu gibi yapıştırabilirsin.
          </p>

          {#if !bulkTarget}
            <!--
              HEDEF BELİRSİZSE EKLEME KAPALI. "Bu satırları hangi sınıfa
              yazayım?" sorusunun çoklu seçimde tek cevabı yok; sessizce
              ilkine yazmak öğretmenin listesini yanlış sınıfa boşaltırdı.
            -->
            <p class="mt-2 text-sm text-body-subtle">
              {selectedIds.length === 0
                ? "Önce bir sınıf seç."
                : "Birden çok sınıf seçili — toplu ekleme için tek sınıf seç (soldaki Temizle işini görür)."}
            </p>
          {/if}
          <Textarea
            rows={10}
            class="mt-3 font-mono text-xs"
            bind:value={bulkText}
            placeholder={"101, Ayşe Yılmaz\n102, Mehmet Demir"}
          />
          <div class="mt-3 flex items-center gap-2">
            <Button size="sm" disabled={busy || !bulkTarget || parsed.length === 0} onclick={addBulk}>
              {parsed.length} öğrenci ekle
            </Button>
            {#if bulkText.trim() !== "" && parsed.length === 0}
              <!--
                Kırmızı elle yazılı: Flowbite'ın tehlike ölçeği rose, bu
                uygulamanınki red (app.css). `text-fg-danger` yazmak uyarının
                rengini değerlendirme/hata için ayrılmış kırmızıdan kaydırırdı.
              -->
              <span class="text-xs text-red-600 dark:text-red-500">Hiçbir satır okunamadı.</span>
            {/if}
          </div>
        </div>
      </aside>
    </div>
  </div>

  <NewClassModal
    bind:open={newClassOpen}
    {busy}
    error={newClassError}
    oncreate={createClass}
  />

  <!--
    SİLME ONAYI. `dismissable={false}` + kendi başlık parçacığımız: bileşenin
    kendi kapatma düğmesinin `aria-label`i kaynağa gömülü İngilizce "Close" ve
    dışarıdan değiştirilemiyor.
  -->
  <Modal
    open={pendingDelete !== null}
    size="xs"
    dismissable={false}
    onclose={() => (pendingDelete = null)}
  >
    {#snippet header()}
      <div class="flex w-full items-center justify-between">
        <h3 class="text-lg font-semibold text-heading">Öğrenciyi sil</h3>
        <Button
          color="alternative"
          class="border-0 p-1.5"
          aria-label="Kapat"
          disabled={busy}
          onclick={() => (pendingDelete = null)}
        >
          <CloseOutline class="h-5 w-5" />
        </Button>
      </div>
    {/snippet}

    {#if pendingDelete}
      <p class="text-sm text-body-subtle">
        <span class="font-semibold text-heading"
          >{pendingDelete.number} — {pendingDelete.first_name} {pendingDelete.last_name}</span
        >
        adlı öğrenci
        <span class="font-semibold text-heading"
          >{classNameById[pendingDelete.classroom_id] ?? "sınıfsız"}</span
        >
        sınıfından silinecek. Bu işlem geri alınamaz.
      </p>
    {/if}

    {#snippet footer()}
      <div class="flex w-full justify-end gap-2">
        <Button color="alternative" disabled={busy} onclick={() => (pendingDelete = null)}>
          Vazgeç
        </Button>
        <Button color="red" disabled={busy} onclick={confirmDelete}>Sil</Button>
      </div>
    {/snippet}
  </Modal>
</PageShell>
