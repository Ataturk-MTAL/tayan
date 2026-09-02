use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;

use tayan_core::{
    application::{
        commands::{
            AddClassicQuestion, AddFillInBlankQuestion,
            AddMultipleChoiceQuestion, AddTrueFalseQuestion,
        },
        ports::{ExamRepository, QuestionBankRepository},
    },
    domain::exam_management::{
        entities::{
            classic::ClassicQuestion,
            fill_in_blank::FillInBlankQuestion,
            multiple_choice::MultipleChoiceQuestion,
            question::{Points, Question, QuestionId},
            true_false::TrueFalseQuestion,
        },
        value_objects::OutcomeCode,
    },
};
use crate::state::AppState;

#[tauri::command]
pub async fn add_multiple_choice_question(
    state:   State<'_, Mutex<AppState>>,
    payload: AddMultipleChoiceQuestion,
) -> Result<String, String> {
    let st = state.lock().await;
    let mut bank = st.bank.load().await.map_err(|e| e.to_string())?;

    let q = Question::MultipleChoice(MultipleChoiceQuestion {
        id:       QuestionId::new(),
        points:   Points::new(payload.points),
        outcomes: payload.outcomes.into_iter()
            .map(|s| OutcomeCode::new(s).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?,
        meta:     payload.meta,
        body:     payload.body,
        options:  payload.options,
        shuffle:  payload.shuffle,
        stats:    Default::default(),
    });
    q.validate().map_err(|e| e.to_string())?;
    let id = q.id().0.to_string();
    bank.add_question(q).map_err(|e| e.to_string())?;
    st.bank.save(&bank).await.map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn add_true_false_question(
    state:   State<'_, Mutex<AppState>>,
    payload: AddTrueFalseQuestion,
) -> Result<String, String> {
    let st = state.lock().await;
    let mut bank = st.bank.load().await.map_err(|e| e.to_string())?;

    let id = QuestionId::new();
    let outcomes = payload.outcomes.into_iter()
        .map(|s| OutcomeCode::new(s).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let q = Question::TrueFalse(TrueFalseQuestion::new(
        id.clone(),
        Points::new(payload.points),
        outcomes,
        payload.body,
        payload.correct_answer,
        payload.meta,
    ));
    q.validate().map_err(|e| e.to_string())?;
    let qid = q.id().0.to_string();
    bank.add_question(q).map_err(|e| e.to_string())?;
    st.bank.save(&bank).await.map_err(|e| e.to_string())?;
    Ok(qid)
}

#[tauri::command]
pub async fn add_fill_in_blank_question(
    state:   State<'_, Mutex<AppState>>,
    payload: AddFillInBlankQuestion,
) -> Result<String, String> {
    let st = state.lock().await;
    let mut bank = st.bank.load().await.map_err(|e| e.to_string())?;

    let outcomes = payload.outcomes.into_iter()
        .map(|s| OutcomeCode::new(s).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let q = Question::FillInBlank(FillInBlankQuestion {
        id:       QuestionId::new(),
        outcomes,
        meta:     payload.meta,
        body:     payload.body,
        blanks:   payload.blanks,
        stats:    Default::default(),
    });
    q.validate().map_err(|e| e.to_string())?;
    let qid = q.id().0.to_string();
    bank.add_question(q).map_err(|e| e.to_string())?;
    st.bank.save(&bank).await.map_err(|e| e.to_string())?;
    Ok(qid)
}

#[tauri::command]
pub async fn add_classic_question(
    state:   State<'_, Mutex<AppState>>,
    payload: AddClassicQuestion,
) -> Result<String, String> {
    let st = state.lock().await;
    let mut bank = st.bank.load().await.map_err(|e| e.to_string())?;

    let outcomes = payload.outcomes.into_iter()
        .map(|s| OutcomeCode::new(s).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let q = Question::Classic(ClassicQuestion {
        id:            QuestionId::new(),
        points:        Points::new(payload.points),
        outcomes,
        meta:          payload.meta,
        body:          payload.body,
        sample_answer: None,
        rubric:        payload.rubric,
        answer_space:  payload.answer_space,
        stats:         Default::default(),
    });
    q.validate().map_err(|e| e.to_string())?;
    let qid = q.id().0.to_string();
    bank.add_question(q).map_err(|e| e.to_string())?;
    st.bank.save(&bank).await.map_err(|e| e.to_string())?;
    Ok(qid)
}

#[tauri::command]
pub async fn list_questions(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<Question>, String> {
    let st   = state.lock().await;
    let bank = st.bank.load().await.map_err(|e| e.to_string())?;
    Ok(bank.questions.into_iter().map(|bq| bq.question).collect())
}

#[tauri::command]
pub async fn delete_question(
    state:       State<'_, Mutex<AppState>>,
    question_id: String,
) -> Result<(), String> {
    let st = state.lock().await;
    let id = QuestionId(Uuid::parse_str(&question_id).map_err(|e| e.to_string())?);

    // Soru bir sınavda kullanılıyorsa silme REDDEDİLİR.
    //
    // Alternatif, sınavlardan atfı sessizce çıkarmaktı. Daha kötü: yayımlanmış
    // bir sınavın içeriği öğretmen görmeden değişir, toplam puan düşer ve kâğıt
    // beklenenden eksik basılır. Silmeyi reddedip hangi sınavlarda kullanıldığını
    // söylemek, kararı öğretmene bırakır.
    //
    // Bunun engellenmediği durumda ne olduğu görülebilir: sınav kaydı bankada
    // karşılığı olmayan bir kimliğe atıf yapar ve arayüz onu "bankada yok" diye
    // göstermek zorunda kalır.
    let exams = st.exams.list(0, u32::MAX).await.map_err(|e| e.to_string())?;
    let kullanan: Vec<String> = exams
        .iter()
        .filter(|exam| exam.questions.iter().any(|q| q.question_id == id))
        .map(|exam| exam.meta.title.clone())
        .collect();

    if !kullanan.is_empty() {
        return Err(format!(
            "Bu soru {} sınavda kullanılıyor: {}. Önce sınavlardan çıkar, sonra sil.",
            kullanan.len(),
            kullanan.join(", ")
        ));
    }

    let mut bank = st.bank.load().await.map_err(|e| e.to_string())?;
    bank.remove_question(&id).map_err(|e| e.to_string())?;
    st.bank.save(&bank).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn update_question(
    state:   State<'_, Mutex<AppState>>,
    payload: Question,
) -> Result<(), String> {
    payload.validate().map_err(|e| e.to_string())?;
    let st  = state.lock().await;
    let mut bank = st.bank.load().await.map_err(|e| e.to_string())?;
    let id  = payload.id().clone();
    let bq  = bank.find_mut(&id)
        .ok_or_else(|| format!("Soru bulunamadı: {id}"))?;
    bq.question = payload;
    st.bank.save(&bank).await.map_err(|e| e.to_string())?;
    Ok(())
}
