#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod store;

use astro_core::data::HOUSE_SYSTEMS;
use astro_core::model::{Person, Place, Settings};
use astro_core::places::{self, Found};
use astro_core::views;
use std::sync::atomic::{AtomicBool, Ordering};
use store::{Data, Store};
use tauri::{AppHandle, Emitter, Manager, PhysicalSize, State, WebviewWindow, WindowEvent};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_updater::UpdaterExt;

type R<T> = Result<T, String>;

/// True while the UI holds people that are not in the file, so closing the window has to ask first.
#[derive(Default)]
struct Unsaved(AtomicBool);

#[tauri::command]
fn set_unsaved(unsaved: bool, state: State<Unsaved>) {
    state.0.store(unsaved, Ordering::Relaxed);
}

/// Closes without raising CloseRequested again; the UI calls this after the user answered.
#[tauri::command]
fn close_app(window: WebviewWindow) -> R<()> {
    window.destroy().map_err(|e| e.to_string())
}

#[tauri::command]
fn search_places(query: String) -> Vec<Found> {
    places::search(&query, 12)
}

#[tauri::command]
fn load_data(store: State<Store>) -> Data {
    store.get()
}

#[tauri::command]
fn save_person(mut person: Person, store: State<Store>) -> R<Data> {
    views::radix(&person, &Settings::default())?; // refuses data that cannot be calculated
    store.update(|d| {
        if person.id.is_empty() {
            person.id = format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
            d.people.push(person.clone());
        } else if let Some(p) = d.people.iter_mut().find(|p| p.id == person.id) {
            *p = person.clone();
        }
        d.people.sort_by_key(|p| p.name.to_lowercase());
    })
}

#[tauri::command]
fn delete_person(id: String, store: State<Store>) -> R<Data> {
    store.update(|d| d.people.retain(|p| p.id != id))
}

#[tauri::command]
fn save_settings(settings: Settings, store: State<Store>) -> R<Data> {
    store.update(|d| d.settings = settings)
}

/// Zooms the whole window, so the layout reflows as with a bigger font; printing keeps its own size.
#[tauri::command]
fn set_zoom(scale: u16, window: WebviewWindow) -> R<()> {
    window.set_zoom(zoom_factor(scale)).map_err(|e| e.to_string())
}

fn zoom_factor(scale: u16) -> f64 {
    f64::from(scale.clamp(80, 200)) / 100.0
}

#[tauri::command]
fn save_sky_place(place: Place, store: State<Store>) -> R<Data> {
    store.update(|d| d.sky_place = Some(place))
}

/// Number of automatic copies and the newest one, as its date.
#[tauri::command]
fn backup_info(store: State<Store>) -> (usize, Option<String>) {
    let copies = store.copies();
    let newest = copies.first().map(|n| n.trim_start_matches("lide-").trim_end_matches(".json").to_string());
    (copies.len(), newest)
}

/// Saves a backup where the user chooses; None when the dialog is cancelled.
#[tauri::command]
async fn export_backup(app: AppHandle, store: State<'_, Store>) -> R<Option<usize>> {
    let name = format!("kosmogram-zaloha-{}.json", chrono::Local::now().format("%Y-%m-%d"));
    let Some(path) = app.dialog().file().set_title("Uložit zálohu lidí").set_file_name(name).add_filter("Záloha Kosmogramu", &["json"]).blocking_save_file() else { return Ok(None) };
    store.export(path.as_path().ok_or("Tuto složku nejde použít.")?).map(Some)
}

/// Adds people from a chosen backup; None when the dialog is cancelled.
#[tauri::command]
async fn import_backup(app: AppHandle, store: State<'_, Store>) -> R<Option<(Data, usize)>> {
    let dir = store.copies_dir();
    let dialog = app.dialog().file().set_title("Vyberte zálohu lidí").add_filter("Záloha Kosmogramu", &["json"]);
    let dialog = if dir.exists() { dialog.set_directory(dir) } else { dialog };
    let Some(path) = dialog.blocking_pick_file() else { return Ok(None) };
    store.import(path.as_path().ok_or("Tento soubor nejde otevřít.")?).map(Some)
}

#[tauri::command]
fn house_systems() -> Vec<(char, &'static str)> {
    HOUSE_SYSTEMS.to_vec()
}

#[tauri::command]
async fn radix(person: Person, settings: Settings) -> R<views::ChartPanel> {
    views::radix(&person, &settings)
}

#[tauri::command]
async fn sky(place: Place, date: String, time: String, settings: Settings) -> R<views::ChartPanel> {
    views::sky(&place, &date, &time, &settings)
}

#[tauri::command]
async fn transits(person: Person, date: String, time: String, settings: Settings) -> R<views::ChartPanel> {
    views::transits(&person, &date, &time, &settings)
}

#[tauri::command]
async fn progressions(person: Person, date: String, settings: Settings) -> R<views::ChartPanel> {
    views::progressions(&person, &date, &settings)
}

#[tauri::command]
async fn events(person: Person, year: i32, month: u32, progression: bool, settings: Settings) -> R<Vec<views::Event>> {
    views::events(&person, year, month, progression, &settings)
}

#[tauri::command]
async fn solar_return(person: Person, year: i32, place: Option<Place>, settings: Settings) -> R<views::ChartPanel> {
    views::solar_return(&person, year, place.as_ref(), &settings)
}

#[tauri::command]
async fn lunar_return(person: Person, from: String, place: Option<Place>, settings: Settings) -> R<(views::ChartPanel, String)> {
    views::lunar_return(&person, &from, place.as_ref(), &settings)
}

#[tauri::command]
fn name_forms(name: String, sex: String) -> astro_core::names::Name {
    astro_core::names::name_of(&name, &sex)
}

#[tauri::command]
async fn relation(a: Person, b: Person, settings: Settings) -> R<views::Relation> {
    views::relation(&a, &b, &settings)
}

#[derive(serde::Serialize)]
struct UpdateInfo {
    version: String,
}

/// Asks the release page whether a newer version exists. None when there is none or there is no internet.
#[tauri::command]
async fn check_update(app: AppHandle) -> Option<UpdateInfo> {
    let update = app.updater().ok()?.check().await.ok()??;
    Some(UpdateInfo { version: update.version })
}

/// Downloads the newer installer, checks its signature and runs it. The installer closes the program,
/// installs over it and starts it again, people and settings stay where they are.
#[tauri::command]
async fn install_update(app: AppHandle, unsaved: State<'_, Unsaved>) -> R<()> {
    if unsaved.0.load(Ordering::Relaxed) {
        return Err("Před aktualizací uložte lidi, které máte jen zobrazené, jinak by se ztratili.".into());
    }
    let failed = |e: tauri_plugin_updater::Error| format!("Aktualizace se nepovedla, zkuste to později. Podrobnosti: {e}");
    let update = app.updater().map_err(failed)?.check().await.map_err(failed)?.ok_or("Novější verze už není k dispozici.")?;
    update.download_and_install(|_, _| {}, || {}).await.map_err(failed)?;
    app.restart()
}

/// The default size can be larger than a small or highly scaled screen; shrink to fit, then show.
fn fit_window(app: &AppHandle) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window("main") else { return Ok(()) };
    if let Some(monitor) = window.current_monitor()? {
        let (screen, now) = (monitor.size(), window.outer_size()?);
        let (max_w, max_h) = ((screen.width as f64 * 0.94) as u32, (screen.height as f64 * 0.88) as u32);
        if now.width > max_w || now.height > max_h {
            window.set_size(PhysicalSize::new(now.width.min(max_w), now.height.min(max_h)))?;
            window.center()?;
        }
    }
    window.show()
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data)?;
            let store = Store::open(data.join("lide.json"));
            let scale = store.get().settings.font_scale;
            app.manage(store);
            if let Some(window) = app.get_webview_window("main") {
                window.set_zoom(zoom_factor(scale))?;
            }
            app.manage(Unsaved::default());
            std::thread::spawn(places::preload);
            fit_window(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.state::<Unsaved>().0.load(Ordering::Relaxed) {
                    api.prevent_close();
                    let _ = window.emit("ask-close", ());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            set_unsaved, close_app, search_places, load_data, save_person, delete_person, save_settings, set_zoom, save_sky_place, house_systems,
            sky, radix, transits, progressions, events, relation, name_forms, solar_return, lunar_return, backup_info, export_backup, import_backup,
            check_update, install_update
        ])
        .run(tauri::generate_context!())
        .expect("spuštění aplikace selhalo");
}
