use anyhow::anyhow;
use anyhow::Result;
use log::debug;
use serde::de::Error;
use serde::Deserialize;
use std::process::Command;
use std::str::FromStr;

pub fn add_exclusion(path: &str) -> Result<()> {
    let output = Command::new("tmutil")
        .arg("addexclusion")
        .arg(path)
        .output()?;

    if !output.status.success() {
        debug!("{:?}", String::from_utf8(output.stderr)?);
        return Err(anyhow!("Failed to add exclusion for {}", path));
    }

    Ok(())
}

pub fn remove_exclusion(path: &str) -> Result<()> {
    let output = Command::new("tmutil")
        .arg("removeexclusion")
        .arg(path)
        .output()?;

    assert!(output.status.success());

    Ok(())
}

#[derive(Debug)]
pub enum CompareArgs {
    Current,
    Backups { first: String, second: String },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CompareInfoChangeItem {
    path: String,
    size: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CompareInfoChangeDifference {
    Size,
}

impl FromStr for CompareInfoChangeDifference {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "size" => Ok(Self::Size),
            _ => Err(()),
        }
    }
}

impl<'de> Deserialize<'de> for CompareInfoChangeDifference {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;

        let value = CompareInfoChangeDifference::from_str(&string);

        match value {
            Ok(value) => Ok(value),
            Err(_) => Err(serde::de::Error::custom(format!(
                "Unknown CompareInfoChangeDifference {}",
                string,
            ))),
        }
    }
}

#[derive(Debug)]
pub enum CompareInfoChange {
    RemovedItem(CompareInfoChangeItem),
    AddedItem(CompareInfoChangeItem),
    RemovedVolume(CompareInfoChangeItem),
    ModifiedItem {
        older: CompareInfoChangeItem,
        newer: CompareInfoChangeItem,
        differences: Vec<CompareInfoChangeDifference>,
    },
}

impl<'de> Deserialize<'de> for CompareInfoChange {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "PascalCase")]
        pub struct ProtoCompareInfoChange {
            removed_volume: Option<CompareInfoChangeItem>,
            differences: Option<Vec<CompareInfoChangeDifference>>,
            newer_item: Option<CompareInfoChangeItem>,
            older_item: Option<CompareInfoChangeItem>,
            added_item: Option<CompareInfoChangeItem>,
            removed_item: Option<CompareInfoChangeItem>,
        }

        let change = ProtoCompareInfoChange::deserialize(deserializer)?;

        if let Some(item) = change.added_item {
            return Ok(CompareInfoChange::AddedItem(item));
        }

        if let Some(item) = change.removed_item {
            return Ok(CompareInfoChange::RemovedItem(item));
        }

        if let Some(item) = change.removed_volume {
            return Ok(CompareInfoChange::RemovedVolume(item));
        }

        let older = change
            .older_item
            .ok_or(D::Error::custom(format!("Missing OlderItem")))?;

        let newer = change
            .newer_item
            .ok_or(D::Error::custom(format!("Missing NewerItem")))?;

        let differences = change
            .differences
            .ok_or(D::Error::custom(format!("Missing Differences")))?;

        Ok(CompareInfoChange::ModifiedItem {
            older,
            newer,
            differences,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CompareInfoTotals {
    added_size: usize,
    changed_size: usize,
    removed_size: usize,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CompareInfo {
    changes: Vec<CompareInfoChange>,
    totals: CompareInfoTotals,
}

fn read_compare_info(str: &str) -> Result<CompareInfo> {
    let result = plist::from_bytes(&str.as_bytes())?;
    Ok(result)
}

pub fn compare(args: &CompareArgs) -> Result<CompareInfo> {
    let mut cmd = &mut Command::new("tmutil");
    cmd = cmd.arg("compare").arg("-sX");

    match args {
        CompareArgs::Current => {}
        CompareArgs::Backups { first, second } => cmd = cmd.args([first, second]),
    }

    let output = cmd.output()?;

    if !output.status.success() {
        debug!("{:?}", String::from_utf8(output.stderr)?);
        return Err(anyhow!("Failed to execute compare"));
    }

    let xml_str = String::from_utf8(output.stdout)?;

    let compare_info = read_compare_info(&xml_str)?;

    dbg!(&compare_info);

    Ok(compare_info)
}

pub fn list_backups() -> Result<Vec<String>> {
    let backups = Command::new("tmutil")
        .arg("listbackups")
        .output()
        .expect("Failed to execute tmutil");

    let backups = String::from_utf8(backups.stdout)?;

    let backup_paths = backups
        .split("\n")
        .filter(|p| !p.trim().is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    Ok(backup_paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializing_compare_xml() {
        let example_str = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
            <plist version="1.0">
                <dict>
                    <key>Changes</key>
                    <array>
                        <dict>
                            <key>RemovedVolume</key>
                            <dict>
                                <key>Path</key>
                                <string>/Volumes/.timemachine/CADA72EA-9283-41D2-BCC2-D0D99A92AEE0/2023-04-29-070108.backup/2023-04-29-070108.backup/Someharddrive</string>
                                <key>Size</key>
                                <integer>923139010427</integer>
                            </dict>
                        </dict>
                        <dict>
                            <key>AddedItem</key>
                            <dict>
                                <key>Path</key>
                                <string>/Library/Apple/System/Library/Receipts/com.apple.pkg.XProtectPayloads_10_15.16U4246.bom</string>
                                <key>Size</key>
                                <integer>60643</integer>
                            </dict>
                        </dict>
                        <dict>
                            <key>RemovedItem</key>
                            <dict>
                                <key>Path</key>
                                <string>/Volumes/.timemachine/CADA72EA-9283-41D2-BCC2-D0D99A92AEE0/2023-04-29-070108.backup/2023-04-29-070108.backup/Macintosh HD - Data/Icon\r</string>
                                <key>Size</key>
                                <integer>0</integer>
                            </dict>
                        </dict>
                        <dict>
                            <key>Differences</key>
                            <array>
                                <string>size</string>
                            </array>
                            <key>NewerItem</key>
                            <dict>
                                <key>Path</key>
                                <string>/Library/Apple/System/Library/CoreServices/XProtect.app/Contents/MacOS/XProtectRemediatorAdload</string>
                                <key>Size</key>
                                <integer>2362672</integer>
                            </dict>
                            <key>OlderItem</key>
                            <dict>
                                <key>Path</key>
                                <string>/Volumes/.timemachine/CADA72EA-9283-41D2-BCC2-D0D99A92AEE0/2023-04-29-070108.backup/2023-04-29-070108.backup/Macintosh HD - Data/Library/Apple/System/Library/CoreServices/XProtect.app/Contents/MacOS/XProtectRemediatorAdload</string>
                                <key>Size</key>
                                <integer>2362608</integer>
                            </dict>
                        </dict>
                    </array>
                    <key>Totals</key>
                    <dict>
                        <key>AddedSize</key>
                        <integer>406080329</integer>
                        <key>ChangedSize</key>
                        <integer>217331952</integer>
                        <key>RemovedSize</key>
                        <integer>855240312822</integer>
                    </dict>
                </dict>
            </plist>
        "#;

        let compare_info: CompareInfo = read_compare_info(&example_str).unwrap();

        assert_eq!(compare_info.totals.added_size, 406080329);
        assert_eq!(compare_info.totals.changed_size, 217331952);
        assert_eq!(compare_info.totals.removed_size, 855240312822);

        assert_eq!(compare_info.changes.len(), 4);

        assert!(matches!(
            compare_info.changes[0],
            CompareInfoChange::RemovedVolume(..)
        ));

        assert!(matches!(
            compare_info.changes[1],
            CompareInfoChange::AddedItem(..)
        ));

        assert!(matches!(
            compare_info.changes[2],
            CompareInfoChange::RemovedItem(..)
        ));

        assert!(matches!(
            &compare_info.changes[3],
            CompareInfoChange::ModifiedItem { .. }
        ));

        // TODO: Find nicer way
        if let CompareInfoChange::ModifiedItem {
            differences,
            older: _,
            newer: _,
        } = &compare_info.changes[3]
        {
            assert_eq!(differences, &vec![CompareInfoChangeDifference::Size])
        } else {
            assert!(false)
        };
    }
}
