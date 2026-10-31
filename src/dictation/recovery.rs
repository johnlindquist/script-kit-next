//! Private recovery journal. The history index is public to authorized readers; this
//! database is not. Use bound IDs, foreign keys and conditional updates: no callback
//! may create a missing (forgotten/expired) take. Audio is committed off the UI thread.
use super::types::CapturedAudioChunk;
use anyhow::{ensure, Context, Result};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

const TTL: i64 = 24 * 60 * 60;
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
fn path() -> PathBuf {
    crate::setup::get_kit_path().join("dictation/recovery/takes.sqlite")
}
fn open(path: &Path) -> Result<Connection> {
    if let Some(owned) = crate::runtime_policy::owned_evaluation() {
        owned.require_owned_path(path)?;
    }
    crate::atomic_file::ensure_private_directory(
        path.parent().context("missing recovery folder")?,
    )?;
    let db = crate::utils::db_permissions::open_private_sqlite(path)?;
    db.busy_timeout(std::time::Duration::from_secs(2))?;
    db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA secure_delete=ON; PRAGMA synchronous=FULL;
      CREATE TABLE IF NOT EXISTS take(id TEXT PRIMARY KEY, started INTEGER NOT NULL, expires INTEGER, state TEXT NOT NULL, raw TEXT, draft TEXT, rev INTEGER NOT NULL DEFAULT 0);
      CREATE TABLE IF NOT EXISTS audio(take TEXT NOT NULL REFERENCES take(id) ON DELETE CASCADE, seq INTEGER NOT NULL, wav BLOB NOT NULL, PRIMARY KEY(take,seq));
      CREATE TABLE IF NOT EXISTS candidate(take TEXT NOT NULL REFERENCES take(id) ON DELETE CASCADE, seq INTEGER PRIMARY KEY, words TEXT NOT NULL);")?;
    db.execute("DELETE FROM take WHERE expires<=?", [now()])?;
    Ok(db)
}
#[derive(Clone, Debug)]
pub struct TakeId {
    pub id: String,
    db: PathBuf,
}
#[derive(Clone, Debug)]
pub struct Item {
    pub id: String,
    pub started: i64,
    pub expires: Option<i64>,
    pub state: String,
    pub raw: Option<String>,
    pub draft: Option<String>,
    pub rev: i64,
    pub has_audio: bool,
}
const SELECT:&str="SELECT id,started,expires,state,raw,draft,rev,EXISTS(SELECT 1 FROM audio WHERE audio.take=t.id) FROM take t";
fn item(row: &rusqlite::Row<'_>) -> rusqlite::Result<Item> {
    Ok(Item {
        id: row.get(0)?,
        started: row.get(1)?,
        expires: row.get(2)?,
        state: row.get(3)?,
        raw: row.get(4)?,
        draft: row.get(5)?,
        rev: row.get(6)?,
        has_audio: row.get(7)?,
    })
}
pub fn purge_expired() -> Result<()> {
    let p = path();
    if p.try_exists()? {
        open(&p)?;
    }
    Ok(())
}
pub fn recent() -> Result<Vec<Item>> {
    let p = path();
    if !p.try_exists()? {
        return Ok(vec![]);
    }
    let db = open(&p)?;
    let mut stmt = db.prepare(&format!(
        "{SELECT} WHERE expires IS NOT NULL ORDER BY started DESC,rowid DESC"
    ))?;
    let items = stmt.query_map([], item)?.collect::<rusqlite::Result<_>>()?;
    Ok(items)
}
impl TakeId {
    pub fn begin() -> Result<Self> {
        Self::begin_at(path())
    }
    fn begin_at(db: PathBuf) -> Result<Self> {
        let id = format!("dictation-{}", uuid::Uuid::new_v4());
        let t = now();
        open(&db)?.execute(
            "INSERT INTO take(id,started,expires,state) VALUES(?,?,?,'interrupted')",
            params![id, t, t + TTL],
        )?;
        Ok(Self { id, db })
    }
    pub fn existing(id: &str) -> Self {
        Self {
            id: id.into(),
            db: path(),
        }
    }
    pub fn get(&self) -> Result<Item> {
        ensure!(self.db.try_exists()?, "No saved recording");
        Ok(open(&self.db)?.query_row(&format!("{SELECT} WHERE id=?"), [&self.id], item)?)
    }
    pub fn cancel(&self, partial: &str) -> Result<()> {
        ensure!(open(&self.db)?.execute("UPDATE take SET state='cancelled',expires=?,draft=coalesce(draft,nullif(?,'')),rev=rev+1 WHERE id=? AND state IN ('interrupted','processing','ready')",params![now()+TTL,partial,self.id])?==1,"Recording is no longer cancellable");
        Ok(())
    }
    pub fn ready(&self, raw: &str, text: &str) -> Result<()> {
        ensure!(open(&self.db)?.execute("UPDATE take SET raw=?,draft=?,state='ready',rev=rev+1 WHERE id=? AND state IN ('interrupted','processing')",params![raw,text,self.id])?==1,"Recording was cancelled, expired or forgotten");
        Ok(())
    }
    pub fn dispatching(&self) -> Result<()> {
        ensure!(
            open(&self.db)?.execute(
                "UPDATE take SET state='delivery unknown' WHERE id=? AND state='ready'",
                [&self.id]
            )? == 1,
            "Recording is no longer ready"
        );
        Ok(())
    }
    pub fn finish(&self) -> Result<()> {
        let mut db = open(&self.db)?;
        let tx = db.transaction()?;
        ensure!(tx.execute("UPDATE take SET state='finished',expires=NULL WHERE id=? AND state IN ('delivery unknown','ready','interrupted','cancelled','withheld','finished') AND draft IS NOT NULL",[&self.id])?==1,"No draft to keep");
        tx.execute("DELETE FROM audio WHERE take IN (SELECT id FROM take WHERE expires IS NULL ORDER BY started DESC,rowid DESC LIMIT -1 OFFSET 25)",[])?;
        tx.execute("DELETE FROM take WHERE expires IS NULL AND id IN (SELECT id FROM take WHERE expires IS NULL ORDER BY started DESC,rowid DESC LIMIT -1 OFFSET 200)",[])?;
        tx.commit()?;
        Ok(())
    }
    pub fn append(&self, seq: u32, samples: &[f32], rate: u32) -> Result<()> {
        ensure!(open(&self.db)?.execute("INSERT INTO audio(take,seq,wav) SELECT id,?,? FROM take WHERE id=? AND state IN ('interrupted','cancelled')",params![seq,super::takes::encode_wav(samples,rate),self.id])?==1,"Audio owner expired or forgotten");
        Ok(())
    }
    pub fn chunks(&self) -> Result<Vec<CapturedAudioChunk>> {
        let db = open(&self.db)?;
        let mut stmt = db.prepare("SELECT wav FROM audio WHERE take=? ORDER BY seq")?;
        let mut result = vec![];
        for wav in stmt.query_map([&self.id], |r| r.get::<_, Vec<u8>>(0))? {
            result.push(super::takes::decode_wav(&wav?)?);
        }
        ensure!(!result.is_empty(), "No recoverable audio");
        Ok(result)
    }
    pub fn edit(&self, rev: i64, words: &str) -> Result<i64> {
        ensure!(
            open(&self.db)?.execute(
                "UPDATE take SET draft=?, rev=rev+1 WHERE id=? AND rev=?",
                params![words, self.id, rev]
            )? == 1,
            "Draft changed, expired or forgotten"
        );
        Ok(rev + 1)
    }
    pub fn add_candidate(&self, words: &str) -> Result<()> {
        open(&self.db)?.execute(
            "INSERT INTO candidate(take,words) VALUES(?,?)",
            params![self.id, words],
        )?;
        Ok(())
    }
    pub fn versions(&self) -> Result<Vec<i64>> {
        let db = open(&self.db)?;
        let mut q = db.prepare("SELECT seq FROM candidate WHERE take=? ORDER BY seq DESC")?;
        let items = q
            .query_map([&self.id], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(items)
    }
    pub fn version(&self, seq: i64) -> Result<String> {
        Ok(open(&self.db)?.query_row(
            "SELECT words FROM candidate WHERE take=? AND seq=?",
            params![self.id, seq],
            |r| r.get(0),
        )?)
    }
    pub fn edit_version(&self, seq: i64, previous: &str, words: &str) -> Result<()> {
        ensure!(
            open(&self.db)?.execute(
                "UPDATE candidate SET words=? WHERE take=? AND seq=? AND words=?",
                params![words, self.id, seq, previous]
            )? == 1,
            "Version changed or forgotten"
        );
        Ok(())
    }
    pub fn withheld(&self) -> Result<()> {
        ensure!(
            open(&self.db)?.execute(
                "UPDATE take SET state='withheld' WHERE id=? AND state='delivery unknown'",
                [&self.id]
            )? == 1,
            "No dispatch in progress"
        );
        Ok(())
    }
    #[cfg(test)]
    pub fn latest_candidate(&self) -> Result<String> {
        Ok(open(&self.db)?.query_row(
            "SELECT words FROM candidate WHERE take=? ORDER BY seq DESC LIMIT 1",
            [&self.id],
            |r| r.get(0),
        )?)
    }
    pub fn forget(&self) -> Result<()> {
        if self.db.try_exists()? {
            open(&self.db)?.execute("DELETE FROM take WHERE id=?", [&self.id])?;
        }
        Ok(())
    }
}
/// Writes one second at a time on the audio processor, never in the realtime device callback.
pub struct AudioJournal {
    take: TakeId,
    seq: u32,
    samples: Vec<f32>,
    rate: u32,
    pub failed: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl AudioJournal {
    pub fn new(take: TakeId) -> Self {
        Self {
            take,
            seq: 0,
            samples: vec![],
            rate: 16000,
            failed: Default::default(),
        }
    }
    pub fn push(&mut self, rate: u32, samples: &[f32]) {
        if self.failed.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }
        self.rate = rate;
        self.samples.extend_from_slice(samples);
        if self.samples.len() >= rate as usize {
            self.flush();
        }
    }
    pub fn flush(&mut self) {
        if self.samples.is_empty() {
            return;
        }
        if self
            .take
            .append(self.seq, &self.samples, self.rate)
            .is_err()
        {
            self.failed
                .store(true, std::sync::atomic::Ordering::Release);
        }
        self.seq += 1;
        self.samples.clear();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn audio_survives_unclean_exit_before_stop() {
        const ENV: &str = "SCRIPT_KIT_SYNTHETIC_CRASH_TEST_ROOT";
        if let Some(root) = std::env::var_os(ENV) {
            let root = PathBuf::from(root);
            assert!(root
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("dictation-crash-test-"));
            let take = TakeId::begin_at(root.join("private.sqlite")).unwrap();
            let (send, recv) = std::sync::mpsc::channel();
            let (events, heard) = async_channel::unbounded();
            std::thread::spawn(move || {
                super::super::capture::process_with_journal(
                    recv,
                    events,
                    Default::default(),
                    Some(AudioJournal::new(take)),
                )
            });
            send.send(super::super::types::RawAudioChunk {
                sample_rate_hz: 16000,
                channels: 1,
                samples: vec![0.1; 16000],
            })
            .unwrap();
            let mut n = 0;
            while n < 16000 {
                if let super::super::types::DictationCaptureEvent::Chunk(c) =
                    heard.recv_blocking().unwrap()
                {
                    n += c.samples.len();
                }
            }
            // No stop, flush, destructor, transcription, or paste callback.
            std::process::exit(73);
        }
        let root = tempfile::Builder::new()
            .prefix("dictation-crash-test-")
            .tempdir()
            .unwrap();
        let db = root.path().join("private.sqlite");
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "dictation::recovery::tests::audio_survives_unclean_exit_before_stop",
                "--test-threads=1",
            ])
            .env(ENV, root.path())
            .output()
            .unwrap();
        assert_eq!(child.status.code(), Some(73));
        let id = open(&db)
            .unwrap()
            .query_row("SELECT id FROM take", [], |r| r.get::<_, String>(0))
            .unwrap();
        let recovered = TakeId { id, db };
        assert_eq!(recovered.get().unwrap().state, "interrupted");
        assert_eq!(recovered.chunks().unwrap()[0].samples.len(), 16000);
        assert!(recovered.get().unwrap().draft.is_none());
    }
    #[test]
    fn exact_record_and_version_survive_new_connections() {
        let root = tempfile::tempdir().unwrap();
        let db = root.path().join("r.sqlite");
        let a = TakeId::begin_at(db.clone()).unwrap();
        let b = TakeId::begin_at(db).unwrap();
        a.ready("raw A", "A").unwrap();
        a.dispatching().unwrap();
        assert_eq!(a.get().unwrap().state, "delivery unknown");
        a.withheld().unwrap();
        a.finish().unwrap();
        assert!(a.get().unwrap().expires.is_none());
        a.add_candidate("1").unwrap();
        a.add_candidate("2").unwrap();
        let ids = a.versions().unwrap();
        assert!(b.version(ids[0]).is_err());
        a.edit_version(ids[1], "1", "edit").unwrap();
        assert!(a.edit_version(ids[1], "1", "stale").is_err());
        assert_eq!(a.version(ids[0]).unwrap(), "2");
        assert_eq!(a.get().unwrap().raw.as_deref(), Some("raw A"));
    }
    #[test]
    fn interruption_cancel_retry_forget_and_expiry() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("takes.sqlite");
        let take = TakeId::begin_at(db.clone()).unwrap();
        let mut stream = AudioJournal::new(take.clone());
        stream.push(16000, &vec![0.1; 17000]);
        drop(stream);
        assert_eq!(take.chunks().unwrap()[0].samples.len(), 17000);
        assert_eq!(take.get().unwrap().state, "interrupted");
        take.cancel("partial").unwrap();
        assert!(take.ready("late", "late").is_err());
        let rev = take.get().unwrap().rev;
        take.edit(rev, "my edit").unwrap();
        assert!(take.edit(rev, "stale").is_err());
        take.add_candidate("new attempt").unwrap();
        assert_eq!(take.get().unwrap().draft.as_deref(), Some("my edit"));
        take.forget().unwrap();
        assert!(take.chunks().is_err());
        assert!(take.latest_candidate().is_err());
        assert!(take.add_candidate("late").is_err());
        assert!(take.append(2, &[0.2], 16000).is_err());
        let other = TakeId::begin_at(db.clone()).unwrap();
        open(&db)
            .unwrap()
            .execute("UPDATE take SET expires=0 WHERE id=?", [&other.id])
            .unwrap();
        assert!(other.get().is_err());
    }
}
