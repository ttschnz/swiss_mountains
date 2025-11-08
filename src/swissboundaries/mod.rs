use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use clap::ValueEnum;
use rusqlite::LoadExtensionGuard;
use tokio_rusqlite::Connection;
// Download db (gpkg.zip) from https://www.swisstopo.admin.ch/en/landscape-model-swissboundaries3d
// and run SELECT InitSpatialMetaData() on it

static DB: &[u8] = include_bytes!("swissBOUNDARIES3D_1_5_LV95_LN02.gpkg");

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum RegionType {
    Landesgebiet,
    Kantonsgebiet,
    Bezirksgebiet,
    Hoheitsgebiet,
}
impl std::fmt::Display for RegionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            RegionType::Landesgebiet => "landesgebiet".to_string(),
            RegionType::Kantonsgebiet => "kantonsgebiet".to_string(),
            RegionType::Bezirksgebiet => "bezirksgebiet".to_string(),
            RegionType::Hoheitsgebiet => "hoheitsgebiet".to_string(),
        };
        write!(f, "{}", str)
    }
}
impl RegionType {
    fn get_table_name(&self) -> String {
        match self {
            RegionType::Landesgebiet => "tlm_landesgebiet".to_string(),
            RegionType::Kantonsgebiet => "tlm_kantonsgebiet".to_string(),
            RegionType::Bezirksgebiet => "tlm_bezirksgebiet".to_string(),
            RegionType::Hoheitsgebiet => "tlm_hoheitsgebiet".to_string(),
        }
    }
}

pub fn dump_db_to_tempfile() -> Result<PathBuf> {
    let temp_path = env::temp_dir().join("swissboundaries3d.sqlite");
    let mut buffer = File::create(&temp_path)?;
    buffer.write_all(DB)?;
    Ok(temp_path)
}

pub async fn open_connection(db_path: PathBuf) -> Result<Connection> {
    let conn = Connection::open(db_path).await?;

    // initialise spatial meta data if not done already.
    // this takes a while, so i recommend to do this with the db
    // before including it in the binary
    conn.call(|conn|{
        unsafe {
            let _guard = LoadExtensionGuard::new(conn)?;
            conn.load_extension("mod_spatialite", None::<&str>)?;
        };
        
        conn.execute_batch(
            "
            SELECT CASE
                WHEN NOT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='spatial_ref_sys')
                THEN (SELECT InitSpatialMetaData())
                ELSE 'Table Exists, no action'
            END;
            "
        )?;
        
        Ok(()) 
    }).await?;

    Ok(conn)
}

pub async fn get_region(
    point: (u32, u32),
    region_type: RegionType,
    conn: &Connection,
) -> Result<Vec<String>> {
    let rows = conn
        .call(move |conn| {
            let mut stmt = conn.prepare(
                format!(
                    "
            SELECT *
                FROM {}
                WHERE ST_Intersects(
                    CastAutomagic(geom),
                    MakePoint(?1, ?2, 2056)
                );
            ",
                    region_type.get_table_name()
                )
                .as_str(),
            )?;
            let mut rows = stmt.query((point.0, point.1))?;

            let mut regions = vec![];

            while let Some(row) = rows.next()? {
                let region: String = row.get("name")?;
                regions.push(region);
            }
            Ok(regions)
        })
        .await?;

    Ok(rows)
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn test_query() {
        let path = dump_db_to_tempfile().unwrap();
        let connection = open_connection(path).await.unwrap();
        let regions = get_region((2616381, 1166112), RegionType::Hoheitsgebiet, &connection)
            .await
            .unwrap();
        assert_eq!(
            regions,
            vec!["Reichenbach im Kandertal"],
            "regions of type 'HOHEITSGEBIET' don't match "
        );
        let regions = get_region((2616381, 1166112), RegionType::Bezirksgebiet, &connection)
            .await
            .unwrap();
        assert_eq!(
            regions,
            vec!["Frutigen-Niedersimmental"],
            "regions of type 'BEZIRKSGEBIET' don't match "
        );
        let regions = get_region((2616381, 1166112), RegionType::Kantonsgebiet, &connection)
            .await
            .unwrap();
        assert_eq!(
            regions,
            vec!["Bern"],
            "regions of type 'KANTONSGEBIET' don't match "
        );
        let regions = get_region((2616381, 1166112), RegionType::Landesgebiet, &connection)
            .await
            .unwrap();
        assert_eq!(
            regions,
            vec!["Schweiz"],
            "regions of type 'LANDESGEBIET' don't match "
        );
    }
}
