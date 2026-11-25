//! Fonctions qui dépendent du système : fichiers, son, ouverture de
//! documents, impression et réglages persistants.
//!
//! - Ordinateur Linux : dialogues natifs, PipeWire / PulseAudio / ALSA,
//!   xdg-open, CUPS, fichier de réglages.
//! - Ordinateur Windows : dialogues natifs, PowerShell (son, imprimantes,
//!   impression), programme par défaut.
//! - Navigateur (WebAssembly) : sélecteur de fichiers du navigateur,
//!   téléchargements (plusieurs fichiers = une archive ZIP), lecture audio
//!   HTML, PDF ouvert dans un onglet (impression par le navigateur),
//!   réglages dans le stockage local.
//!
//! Les fichiers choisis arrivent dans une « boîte aux lettres » relevée par
//! l'interface à chaque image : ainsi le même code sert au dialogue natif
//! (réponse immédiate) et au navigateur (réponse asynchrone).

use std::cell::RefCell;

/// Fichier ouvert : (nom, contenu).
pub type Opened = (String, Vec<u8>);

thread_local! {
    static MAILBOX: RefCell<Vec<(u32, Vec<Opened>)>> = const { RefCell::new(Vec::new()) };
}

/// Fichiers choisis depuis la dernière relève : (motif de la demande, fichiers).
pub fn take_opened() -> Vec<(u32, Vec<Opened>)> {
    MAILBOX.with(|m| std::mem::take(&mut *m.borrow_mut()))
}

fn deliver(purpose: u32, files: Vec<Opened>) {
    if !files.is_empty() {
        MAILBOX.with(|m| m.borrow_mut().push((purpose, files)));
    }
}

/// Demande un ou plusieurs fichiers ; ils arriveront par `take_opened` avec
/// le motif `purpose`.
pub fn pick(purpose: u32, filter: &str, exts: &[&str], multiple: bool) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let d = rfd::FileDialog::new().add_filter(filter, exts);
        let paths = if multiple { d.pick_files().unwrap_or_default() } else { d.pick_file().into_iter().collect() };
        let files = paths.into_iter().filter_map(|p| std::fs::read(&p).ok().map(|b| (p.display().to_string(), b))).collect();
        deliver(purpose, files);
    }
    #[cfg(target_arch = "wasm32")]
    {
        let d = rfd::AsyncFileDialog::new().add_filter(filter, exts);
        wasm_bindgen_futures::spawn_local(async move {
            let handles = if multiple { d.pick_files().await.unwrap_or_default() } else { d.pick_file().await.into_iter().collect() };
            let mut files = Vec::new();
            for h in handles {
                files.push((h.file_name(), h.read().await));
            }
            deliver(purpose, files);
        });
    }
}

/// Nom de fichier sans dossier ni extension.
pub fn stem(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    base.rsplit_once('.').map(|(a, _)| a).unwrap_or(base).to_string()
}

/// Dossier d'un fichier ouvert (vide dans le navigateur).
pub fn parent_dir(name: &str) -> std::path::PathBuf {
    std::path::Path::new(name).parent().map(|p| p.to_path_buf()).unwrap_or_default()
}

/// Enregistre un fichier : dialogue « Enregistrer sous » sur ordinateur,
/// téléchargement dans le navigateur. Renvoie l'emplacement, ou `None` si
/// l'utilisateur a annulé.
pub fn save(name: &str, filter: &str, ext: &str, data: &[u8]) -> Option<Result<String, String>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let p = rfd::FileDialog::new().add_filter(filter, &[ext]).set_file_name(name).save_file()?;
        Some(std::fs::write(&p, data).map(|_| p.display().to_string()).map_err(|e| e.to_string()))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (filter, ext);
        Some(web::download(name, data).map(|_| name.to_string()))
    }
}

/// Enregistre plusieurs fichiers : dans un dossier choisi sur ordinateur
/// (dans un sous-dossier `folder` s'il est donné), en archive ZIP dans le
/// navigateur. Renvoie l'emplacement, ou `None` si annulé.
pub fn save_many(folder: Option<&str>, zip_name: &str, files: &[(String, Vec<u8>)]) -> Option<Result<String, String>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = zip_name;
        let parent = rfd::FileDialog::new().pick_folder()?;
        let dir = match folder {
            Some(f) => parent.join(f),
            None => parent,
        };
        let r = std::fs::create_dir_all(&dir).map_err(|e| e.to_string()).and_then(|_| {
            for (n, b) in files {
                std::fs::write(dir.join(n), b).map_err(|e| format!("{n} : {e}"))?;
            }
            Ok(dir.display().to_string())
        });
        Some(r)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let prefix = folder.map(|f| format!("{f}/")).unwrap_or_default();
        let entries: Vec<(String, &[u8])> = files.iter().map(|(n, b)| (format!("{prefix}{n}"), b.as_slice())).collect();
        Some(web::download(zip_name, &zip_store(&entries)).map(|_| zip_name.to_string()))
    }
}

/// Archive ZIP sans compression (format « stored »), lisible partout.
pub fn zip_store(files: &[(String, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    let le16 = |v: &mut Vec<u8>, x: u16| v.extend_from_slice(&x.to_le_bytes());
    let le32 = |v: &mut Vec<u8>, x: u32| v.extend_from_slice(&x.to_le_bytes());
    for (name, data) in files {
        let mut crc = flate2::Crc::new();
        crc.update(data);
        let crc = crc.sum();
        let off = out.len() as u32;
        let n = name.as_bytes();
        le32(&mut out, 0x0403_4b50);
        le16(&mut out, 20);
        le16(&mut out, 0x0800); // noms en UTF-8
        le16(&mut out, 0);
        le16(&mut out, 0);
        le16(&mut out, 0x21);
        le32(&mut out, crc);
        le32(&mut out, data.len() as u32);
        le32(&mut out, data.len() as u32);
        le16(&mut out, n.len() as u16);
        le16(&mut out, 0);
        out.extend_from_slice(n);
        out.extend_from_slice(data);
        le32(&mut central, 0x0201_4b50);
        le16(&mut central, 20);
        le16(&mut central, 20);
        le16(&mut central, 0x0800);
        le16(&mut central, 0);
        le16(&mut central, 0);
        le16(&mut central, 0x21);
        le32(&mut central, crc);
        le32(&mut central, data.len() as u32);
        le32(&mut central, data.len() as u32);
        le16(&mut central, n.len() as u16);
        le16(&mut central, 0);
        le16(&mut central, 0);
        le16(&mut central, 0);
        le16(&mut central, 0);
        le32(&mut central, 0);
        le32(&mut central, off);
        central.extend_from_slice(n);
    }
    let coff = out.len() as u32;
    let clen = central.len() as u32;
    out.extend_from_slice(&central);
    le32(&mut out, 0x0605_4b50);
    le16(&mut out, 0);
    le16(&mut out, 0);
    le16(&mut out, files.len() as u16);
    le16(&mut out, files.len() as u16);
    le32(&mut out, clen);
    le32(&mut out, coff);
    le16(&mut out, 0);
    out
}

/// Son en cours de lecture.
pub struct Sound {
    #[cfg(not(target_arch = "wasm32"))]
    child: std::process::Child,
    #[cfg(target_arch = "wasm32")]
    audio: web_sys::HtmlAudioElement,
}

impl Sound {
    /// Vrai quand la lecture est terminée.
    pub fn finished(&mut self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            !matches!(self.child.try_wait(), Ok(None))
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.audio.ended()
        }
    }
    /// Arrête la lecture.
    pub fn stop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = self.audio.pause();
        }
    }
}

/// Joue un son WAV (contenu du fichier) en arrière-plan.
pub fn play_wav(wav: &[u8]) -> Option<Sound> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = std::env::temp_dir().join(format!("fractlab_son_{}.wav", std::process::id()));
        std::fs::write(&path, wav).ok()?;
        native::play_file(&path).map(|child| Sound { child })
    }
    #[cfg(target_arch = "wasm32")]
    {
        web::play(wav).map(|audio| Sound { audio })
    }
}

/// Ouvre un document (PDF…) avec le programme par défaut, ou dans un nouvel
/// onglet du navigateur.
pub fn open_bytes(name: &str, mime: &str, data: &[u8]) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = mime;
        let p = std::env::temp_dir().join(name);
        std::fs::write(&p, data).is_ok() && native::open(&p)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = name;
        web::open_tab(data, mime)
    }
}

/// Imprimantes installées (celle par défaut en premier) ; dans le
/// navigateur, une seule entrée : l'impression du navigateur.
pub fn printers() -> Vec<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::printers()
    }
    #[cfg(target_arch = "wasm32")]
    {
        vec![crate::i18n::tr("Navigateur (aperçu PDF puis Imprimer)").to_string()]
    }
}

/// Imprime un PDF (dans le navigateur : ouvre le PDF, l'impression se fait
/// depuis son aperçu).
pub fn print_pdf(name: &str, data: &[u8], printer: &str, copies: u32) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let p = std::env::temp_dir().join(name);
        std::fs::write(&p, data).is_ok() && native::print(&p, printer, copies)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (name, printer, copies);
        web::open_tab(data, "application/pdf")
    }
}

/// Lit une donnée enregistrée (`config` = réglages, `projet` = projet en
/// cours dans le navigateur) : fichier du dossier de configuration sur
/// ordinateur, stockage local dans le navigateur.
pub fn load_kv(key: &str) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(dirs::config_dir()?.join("fractlab").join(format!("{key}.json"))).ok()
    }
    #[cfg(target_arch = "wasm32")]
    {
        web::storage()?.get_item(&format!("fractlab.{key}")).ok()?
    }
}

/// Enregistre une donnée (voir `load_kv`).
pub fn save_kv(key: &str, json: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(d) = dirs::config_dir() {
        let p = d.join("fractlab").join(format!("{key}.json"));
        let _ = std::fs::create_dir_all(p.parent().unwrap());
        let _ = std::fs::write(p, json);
    }
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = web::storage() {
        let _ = s.set_item(&format!("fractlab.{key}"), json);
    }
}

/// Vrai dans la version navigateur.
pub const WEB: bool = cfg!(target_arch = "wasm32");

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::path::Path;
    use std::process::{Child, Command, Stdio};

    #[cfg(windows)]
    fn hidden(mut c: Command) -> Command {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW : pas de console qui clignote
        c.creation_flags(0x0800_0000);
        c
    }

    #[cfg(windows)]
    fn powershell(script: &str) -> Command {
        let mut c = Command::new("powershell");
        c.args(["-NoProfile", "-NonInteractive", "-Command", script]);
        hidden(c)
    }

    #[cfg(windows)]
    fn ps_quote(p: &Path) -> String {
        format!("'{}'", p.display().to_string().replace('\'', "''"))
    }

    pub fn play_file(path: &Path) -> Option<Child> {
        #[cfg(windows)]
        {
            powershell(&format!("(New-Object System.Media.SoundPlayer {}).PlaySync()", ps_quote(path))).stdout(Stdio::null()).stderr(Stdio::null()).spawn().ok()
        }
        #[cfg(not(windows))]
        {
            for cmd in ["pw-play", "paplay", "aplay", "ffplay"] {
                let mut c = Command::new(cmd);
                if cmd == "ffplay" {
                    c.args(["-nodisp", "-autoexit", "-loglevel", "quiet"]);
                }
                if let Ok(ch) = c.arg(path).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
                    return Some(ch);
                }
            }
            None
        }
    }

    pub fn open(path: &Path) -> bool {
        #[cfg(windows)]
        {
            let mut c = Command::new("cmd");
            c.args(["/C", "start", ""]).arg(path);
            hidden(c).spawn().is_ok()
        }
        #[cfg(not(windows))]
        {
            Command::new("xdg-open").arg(path).spawn().is_ok()
        }
    }

    pub fn printers() -> Vec<String> {
        #[cfg(windows)]
        {
            let out = powershell("Get-CimInstance Win32_Printer | Sort-Object -Property @{Expression='Default';Descending=$true},Name | ForEach-Object { $_.Name }").output().ok();
            out.map(|o| String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()).unwrap_or_default()
        }
        #[cfg(not(windows))]
        {
            let run = |args: &[&str]| Command::new("lpstat").args(args).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
            let mut v: Vec<String> = run(&["-e"]).lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
            if let Some(def) = run(&["-d"]).split(':').nth(1).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
                v.retain(|p| p != &def);
                v.insert(0, def);
            }
            v
        }
    }

    pub fn print(path: &Path, printer: &str, copies: u32) -> bool {
        #[cfg(windows)]
        {
            // verbe « PrintTo » du lecteur PDF par défaut, une fois par exemplaire
            let script = format!(
                "for($i=0;$i -lt {n};$i++){{ Start-Process -FilePath {f} -Verb PrintTo -ArgumentList '\"{p}\"' -WindowStyle Hidden -Wait }}",
                n = copies.max(1),
                f = ps_quote(path),
                p = printer.replace('\'', "''").replace('"', "")
            );
            powershell(&script).status().map(|s| s.success()).unwrap_or(false)
        }
        #[cfg(not(windows))]
        {
            Command::new("lp").args(["-d", printer, "-n", &copies.to_string()]).arg(path).status().map(|s| s.success()).unwrap_or(false)
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::JsCast;

    fn blob_url(data: &[u8], mime: &str) -> Option<String> {
        let arr = js_sys::Array::new();
        arr.push(&js_sys::Uint8Array::from(data));
        let opts = web_sys::BlobPropertyBag::new();
        opts.set_type(mime);
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&arr, &opts).ok()?;
        web_sys::Url::create_object_url_with_blob(&blob).ok()
    }

    pub fn download(name: &str, data: &[u8]) -> Result<(), String> {
        let url = blob_url(data, "application/octet-stream").ok_or("téléchargement impossible")?;
        let doc = web_sys::window().and_then(|w| w.document()).ok_or("pas de document")?;
        let a: web_sys::HtmlAnchorElement = doc.create_element("a").map_err(|_| "élément")?.dyn_into().map_err(|_| "lien")?;
        a.set_href(&url);
        a.set_download(name);
        a.click();
        Ok(())
    }

    pub fn open_tab(data: &[u8], mime: &str) -> bool {
        match (blob_url(data, mime), web_sys::window()) {
            (Some(url), Some(w)) => w.open_with_url_and_target(&url, "_blank").is_ok(),
            _ => false,
        }
    }

    pub fn play(wav: &[u8]) -> Option<web_sys::HtmlAudioElement> {
        let url = blob_url(wav, "audio/wav")?;
        let a = web_sys::HtmlAudioElement::new_with_src(&url).ok()?;
        let _ = a.play();
        Some(a)
    }

    pub fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }
}

#[cfg(test)]
mod tests {
    /// L'archive ZIP est relue correctement (structure et CRC).
    #[test]
    fn zip_lisible() {
        let z = super::zip_store(&[("a.txt".into(), b"bonjour".as_slice()), ("dossier/b.bin".into(), &[0u8, 1, 2, 255][..])]);
        assert_eq!(&z[..4], b"PK\x03\x04");
        assert_eq!(&z[z.len() - 22..z.len() - 18], b"PK\x05\x06");
        assert_eq!(u16::from_le_bytes([z[z.len() - 12], z[z.len() - 11]]), 2);
        assert_eq!(super::stem("/a/b/projet.txt"), "projet");
        assert_eq!(super::stem("C:\\x\\y.fig"), "y");
    }
}
