//! Enumerate human users (UID ≥ 1000) plus optional avatar paths.

use anyhow::Result;
use novau_common::SystemUser;
use std::path::PathBuf;

pub fn enumerate() -> Result<Vec<SystemUser>> {
    let mut out = Vec::new();
    for line in std::fs::read_to_string("/etc/passwd")?.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 7 {
            continue;
        }
        let name = fields[0].to_string();
        let uid: u32 = match fields[2].parse() {
            Ok(u) => u,
            Err(_) => continue,
        };
        // Skip system accounts
        if uid < 1000 || uid == 65534 {
            continue;
        }
        // Skip nologin / false shells
        let shell = PathBuf::from(fields[6]);
        if let Some(f) = shell.file_name().and_then(|s| s.to_str()) {
            if matches!(f, "nologin" | "false") {
                continue;
            }
        }
        let real_name = fields[4]
            .split(',')
            .next()
            .filter(|s| !s.is_empty())
            .map(String::from);
        let avatar = avatar_for(&name);
        out.push(SystemUser {
            name,
            uid,
            real_name,
            home: PathBuf::from(fields[5]),
            shell,
            avatar_path: avatar,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn avatar_for(user: &str) -> Option<PathBuf> {
    let p = PathBuf::from("/var/lib/AccountsService/icons").join(user);
    if p.exists() {
        Some(p)
    } else {
        None
    }
}
