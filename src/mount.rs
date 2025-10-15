use crate::error::{SarusError, SarusResult};
use serde::{Deserialize, Serialize};

pub type SarusMounts = Vec<SarusMount>;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct SarusMount {
    source: String,
    target: String,
    flags: String,
}

impl SarusMount {
    pub fn try_new(input: String) -> SarusResult<SarusMount> {
        let mut a = input.split(":");
        let asize = a.clone().count();

        if asize < 2 || asize > 3 {
            return Err(SarusError {
                code: 8,
                file_path: None,
                msg: format!(
                    "{} contains {} number of fields, expected 2 or 3",
                    input, asize
                ),
            });
        };

        let mut s = a.next().unwrap();
        let t = a.next().unwrap();
        let mut f = "";
        if asize == 3 {
            f = a.next().unwrap();
        }
        let mut df = "";

        let mut ps: std::path::PathBuf = std::path::Path::new(s).into();
        let pt: std::path::PathBuf = std::path::Path::new(t).into();

        if f == "sqsh" {
            if ps.starts_with(".") {
                ps = match std::path::absolute(&ps) {
                    Err(_) => {
                        return Err(SarusError {
                            code: 9,
                            file_path: None,
                            msg: format!("cannot translate {} in an absolute path", ps.display()),
                        });
                    }
                    Ok(ok) => ok,
                }
            } else if ps.starts_with("/") {
                ()
            } else {
                return Err(SarusError {
                    code: 10,
                    file_path: None,
                    msg: format!(
                        "source of squashfs mount {} must be a relative path or an absolute path starting with . or /",
                        s
                    ),
                });
            }
            s = match ps.as_os_str().to_str() {
                Some(ok) => ok,
                None => {
                    return Err(SarusError {
                        code: 11,
                        file_path: None,
                        msg: format!("cannot translate {} into string", ps.display()),
                    });
                }
            };
        } else {
            if [".", "/"].iter().any(|s| ps.starts_with(*s)) {
                df = "x-create=auto,rbind";
            } else {
                if s == "tmpfs" {
                    df = "x-create=dir";
                } else if s == "umount" {
                    df = "x-detach";
                } else {
                    return Err(SarusError {
                        code: 12,
                        file_path: None,
                        msg: format!(
                            "mount source {ps:#?} must be one among a relative path starting with . , an absolute path starting with / , \"tmpfs\" or \"umount\""
                        ),
                    });
                }
            }
        }

        if ![".", "/"].iter().any(|s| pt.starts_with(*s)) {
            return Err(SarusError {
                code: 13,
                file_path: None,
                msg: format!(
                    "mount target {pt:#?} must be one among a relative path starting with . or an absolute path starting with /"
                ),
            });
        }

        let es = escape_mount(String::from(s));
        let et = escape_mount(String::from(t));

        let em;

        if f == "sqsh" {
            let metadata = match std::fs::metadata(s) {
                Ok(m) => m,
                Err(e) => {
                    return Err(SarusError {
                        code: 14,
                        file_path: None,
                        msg: format!("could not stat source of squashfs mount ({s}): {e}"),
                    });
                }
            };
            if !metadata.is_file() {
                return Err(SarusError {
                    code: 14,
                    file_path: None,
                    msg: format!("source of squashfs mount ({s}) must be a regular file"),
                });
            }

            em = SarusMount {
                source: String::from(es),
                target: String::from(et),
                flags: String::from(""),
            }
        } else {
            let flags;
            if f != "" {
                flags = format!("{df},{f}");
            } else {
                flags = format!("{df}");
            }

            em = SarusMount {
                source: String::from(es),
                target: String::from(et),
                flags: String::from(flags),
            }
        }

        return Ok(em);
    }
}

pub fn sarus_mounts_from_strings(input: Vec<String>) -> SarusResult<SarusMounts> {
    let mut res = vec![];

    for i in input.iter() {
        let m = SarusMount::try_new(i.clone())?;
        if !res.contains(&m) {
            res.push(m.clone());
        }
    }

    Ok(res)
}

/*
impl SarusMounts {
    pub fn merge(&mut self, mut mounts: SarusMounts) -> SarusResult<()> {
        self.list.append(&mut mounts.list);
        Ok(())
    }

    pub fn new(input: Vec<String>) -> SarusMounts {
        let mut new = SarusMounts {
            list: vec![],
        };

        for i in input.iter() {
            let _ = new.add(String::from(i));
        }

        SarusMounts {
            list: new.list,
        }
    }

    pub fn add(&mut self, mount: String) -> SarusResult<()> {
        let mut a = mount.split(":");
        let asize = a.clone().count();

        if asize < 2 || asize > 3 {
            eprintln!("{mount}");
            return Err(
                SarusError {
                    code: 8,
                    file_path: None,
                    msg: format!("{} number of field, expected 2 or 3", asize),
                });
        };

        let mut s = a.next().unwrap();
        let t = a.next().unwrap();
        let mut f = "";
        if asize == 3 {
            f = a.next().unwrap();
        }
        let mut df = "";

        let mut ps: std::path::PathBuf = std::path::Path::new(s).into();
        let pt: std::path::PathBuf = std::path::Path::new(t).into();

        if f == "sqsh" {
            if ps.is_relative() {
                ps = match std::path::absolute(&ps) {
                        Err(_) => {
                            return Err(
                                SarusError {
                                    code: 9,
                                    file_path: None,
                                    msg: format!("cannot translate {} in an absolute path", ps.display()),
                                });
                            },
                        Ok(ok) => ok,
                    }
            } else if ps.is_absolute() {
                ()
            } else {
                return Err(
                    SarusError {
                        code: 10,
                        file_path: None,
                        msg: format!("source of squashfs mount {} must be a relative path or an absolute path", s),
                    });
            }
            s = match ps.as_os_str().to_str() {
                Some(ok) => ok,
                None => { return Err(
                    SarusError {
                        code: 11,
                        file_path: None,
                        msg: format!("cannot translate {} into string", ps.display()),
                    })},
            };
        } else {
            if ps.is_relative() || ps.is_absolute() {
                df = "x-create=auto,rbind";
            } else {
                if s == "tmpfs" {
                    df = "x-create=dir";
                } else if s == "umount" {
                    df = "x-detach";
                } else {
                    return Err(
                        SarusError {
                            code: 12,
                            file_path: None,
                            msg: format!("mount source must be a relative path, an absolute path, \"tmpfs\" or \"umount\""),
                        });
                }
            }

        }

        if ! pt.is_relative() && ! pt.is_absolute() {
            return Err(
                SarusError {
                    code: 13,
                    file_path: None,
                    msg: format!("mount target must be a relative path or an absolute path"),
                });
        }

        let es = escape_mount(String::from(s));
        let et = escape_mount(String::from(t));

        let em;

        if f == "sqsh" {
            let metadata = match std::fs::metadata(s) {
                Ok(m) => m,
                Err(e) => {
                    return Err(
                        SarusError {
                            code: 14,
                            file_path: None,
                            msg: format!("could not stat source of squashfs mount ({s}): {e}"),
                        });
               },
            };
            if ! metadata.is_file() {
                return Err(
                    SarusError {
                        code: 14,
                        file_path: None,
                        msg: format!("source of squashfs mount ({s}) must be a regular file"),
                    });
            }
            em = format!("{es} {et}");
        } else {
            let flags;
            if f != "" {
                flags = format!("{df},{f}");
            } else {
                flags = format!("{df}");
            }
            em = format!("{es} {et} {flags}");
        }

        if ! self.list.contains(&em) {
                self.list.push(em.clone());
        }
        Ok(())
    }
}
*/

// From pyxis code (still needed ???)
// escape source or target mount entry to build an fstab like entry as used by enroot
// from man 3 getmntent:
//     Since fields in the mtab and fstab files are separated by
//     whitespace, octal escapes are used to represent the characters
//     space (\040), tab (\011), newline (\012), and backslash (\\) in
//     those files when they occur in one of the four strings in a
//     mntent structure.  The routines addmntent() and getmntent() will
//     convert from string representation to escaped representation and
//     back.  When converting from escaped representation, the sequence
//     \134 is also converted to a backslash.
fn escape_mount(path: String) -> String {
    let mut epath = String::from("");
    for c1 in path.chars() {
        let c2 = match c1 {
            ' ' => format!("\\040"),
            '\t' => format!("\\011"),
            '\n' => format!("\\012"),
            '\\' => format!("\\\\"),
            _ => format!("{c1}"),
        };
        epath.push_str(c2.as_str());
    }
    epath
}
