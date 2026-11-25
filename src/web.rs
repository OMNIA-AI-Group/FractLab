//! Version navigateur (WebAssembly) : démarrage de l'interface dans une
//! balise `<canvas>`, et Web Workers qui exécutent les calculs de fond pour
//! que l'interface ne gèle jamais.
//!
//! La page fournit `window.__fractlab_worker()`, qui crée un Worker déjà
//! initialisé avec le même module WebAssembly ; le Worker appelle
//! `worker_entry` avec chaque requête reçue et renvoie les réponses encodées.

use crate::bg;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Démarre FractLab dans la balise `<canvas id=canvas_id>`.
#[wasm_bindgen]
pub async fn start(canvas_id: String) -> Result<(), JsValue> {
    let opts = eframe::WebOptions::default();
    eframe::WebRunner::new().start(&canvas_id, opts, Box::new(|cc| Box::new(crate::app::App::new(cc)))).await
}

/// Point d'entrée du Worker : exécute une requête encodée et renvoie chaque
/// réponse encodée par `postMessage`.
#[wasm_bindgen]
pub fn worker_entry(data: &[u8]) {
    let Some(req) = bg::decode::<bg::Request>(data) else { return };
    let scope: web_sys::DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    bg::execute(req, &mut |reply| {
        let b = bg::encode(&reply);
        let _ = scope.post_message(&js_sys::Uint8Array::from(&b[..]));
    });
}

/// Worker d'une tâche : arrêté quand la tâche est abandonnée ou terminée.
/// Vide quand le calcul a été fait directement (navigateur sans Worker).
pub struct WorkerGuard {
    worker: Option<web_sys::Worker>,
    _on_message: Option<Closure<dyn FnMut(web_sys::MessageEvent)>>,
}

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        if let Some(w) = &self.worker {
            w.terminate();
        }
    }
}

/// Lance une requête dans un nouveau Worker ; `on_reply` reçoit chaque réponse.
pub fn worker(req: bg::Request, mut on_reply: impl FnMut(bg::Reply) + 'static) -> WorkerGuard {
    // la page fournit la fabrique si les Workers fonctionnent ; sinon (certains
    // navigateurs en fichier local) le calcul est fait directement dans la page
    let factory = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("__fractlab_worker")).ok().and_then(|f| f.dyn_into::<js_sys::Function>().ok());
    let created = factory.and_then(|f| f.call0(&JsValue::NULL).ok()).and_then(|w| w.dyn_into::<web_sys::Worker>().ok());
    let Some(worker) = created else {
        bg::execute(req, &mut on_reply);
        return WorkerGuard { worker: None, _on_message: None };
    };
    let on_message = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |e: web_sys::MessageEvent| {
        let bytes = js_sys::Uint8Array::new(&e.data()).to_vec();
        if let Some(r) = bg::decode::<bg::Reply>(&bytes) {
            on_reply(r);
        }
    });
    worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    let b = bg::encode(&req);
    let _ = worker.post_message(&js_sys::Uint8Array::from(&b[..]));
    WorkerGuard { worker: Some(worker), _on_message: Some(on_message) }
}
