use rusqlite::{params, Connection};
use std::{thread, time::Duration};
use windows::Win32::{
    Foundation::BOOL,
    System::SystemInformation::GetTickCount64,
    UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
};

fn main() -> rusqlite::Result<()> {
    println!("Program pokrenut – praćenje aktivnosti započeto...");

    let conn = Connection::open("activity.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS inactivity (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time TEXT,
            end_time TEXT,
            duration_seconds INTEGER
        )",
        [],
    )?;

    let now = chrono::Local::now();
    println!("Računalo/servis pokrenut u: {}", now.format("%Y-%m-%d %H:%M:%S"));
    conn.execute(
        "INSERT INTO inactivity (start_time, end_time, duration_seconds)
         VALUES (?1, ?2, ?3)",
        params![now.to_rfc3339(), now.to_rfc3339(), 0],
    )?;

    let mut inactive_start: Option<chrono::DateTime<chrono::Local>> = None;

    loop {
        let idle = get_idle_time();

        if idle > Duration::from_secs(60) {
            if inactive_start.is_none() {
                let now = chrono::Local::now();
                inactive_start = Some(now);
                println!("⏸️  Početak neaktivnosti: {}", now.format("%H:%M:%S"));
            }
        } else {
            if let Some(start_time) = inactive_start.take() {
                let end_time = chrono::Local::now();
                let duration = (end_time - start_time).num_seconds();

                conn.execute(
                    "INSERT INTO inactivity (start_time, end_time, duration_seconds)
                     VALUES (?1, ?2, ?3)",
                    params![start_time.to_rfc3339(), end_time.to_rfc3339(), duration],
                )?;

                println!(
                    "💾 Zapisano u bazu: neaktivnost od {} do {} ({} sekundi)",
                    start_time.format("%H:%M:%S"),
                    end_time.format("%H:%M:%S"),
                    duration
                );

                println!("Korisnik je postao aktivan – program završava.");
                break;
            }
        }

        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

fn get_idle_time() -> Duration {
    unsafe {
        let mut lii = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };

        if GetLastInputInfo(&mut lii as *mut LASTINPUTINFO) == BOOL(0) {
            return Duration::from_secs(0);
        }

        let tick_count = GetTickCount64();
        let idle_ms = tick_count - lii.dwTime as u64;
        Duration::from_millis(idle_ms)
    }
}
