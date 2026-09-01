// Tauri webview'i tek sayfa uygulaması olarak çalışır; sunucu tarafı render yok.
// Bu satır olmadan Tauri IPC çağrıları fallback üretimi sırasında çalışmaya
// kalkar ve derleme kırılır.
export const ssr = false;
export const prerender = false;
