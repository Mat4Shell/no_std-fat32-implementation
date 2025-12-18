use crate::{
    block::BlockDevice,
    volume::Fat32Volume,
    error::FatError,
};
use alloc::vec;
use alloc::vec::Vec;

/// Possible attributes for a FAT32 file or directory.
pub const ATTR_ARCHIVE: u8 = 0x20;

/// Create and write a new file to the FAT32 volume.
pub fn create_file<B: BlockDevice>(
    volume: &mut Fat32Volume<B>,
    dir_cluster: u32,
    filename: &str,
    data: &[u8],
) -> Result<(), FatError> {
    // Number or clusters needed
    let cluster_size = volume.cluster_size() as usize;
    let clusters_needed = (data.len() + cluster_size - 1) / cluster_size;

    // Find free in FAT and allocate clusters
    let free_clusters = find_free_clusters(volume, clusters_needed)?;

    // Write data to allocated clusters
    for (i, &cluster) in free_clusters.iter().enumerate() {
        let start = i * cluster_size;
        let end = core::cmp::min(start + cluster_size, data.len());
        let cluster_data = &data[start..end];
        write_cluster(volume, cluster, cluster_data)?;
    }

    // update FAT entries
    update_fat_entries(volume, &free_clusters)?;

    // add directory entry in root directory
    add_directory_entry(volume, dir_cluster, filename, &free_clusters, data.len() as u32)?;

    Ok(())
}

/// Find free clusters in the FAT.
fn find_free_clusters<B: BlockDevice>(
    volume: &Fat32Volume<B>,
    count: usize,
) -> Result<Vec<u32>, FatError> {
    // browse FAT to find free clusters
    let mut free_clusters = Vec::new();
    let fat_start = volume.boot.reserved_sectors as u64;
    let fat_size = volume.boot.fat_size_sectors as u64;
    let total_clusters = (fat_size * volume.boot.bytes_per_sector as u64) / 4;
    let mut fat_sector = [0u8; 512];
    for cluster in 2..total_clusters {
        let fat_offset = cluster * 4;
        let sector_index = fat_offset / 512;
        let byte_index = (fat_offset % 512) as usize;

        if sector_index >= fat_size {
            break;
        }

        volume.device.read_sector(fat_start + sector_index, &mut fat_sector);
        let entry = u32::from_le_bytes([
            fat_sector[byte_index],
            fat_sector[byte_index + 1],
            fat_sector[byte_index + 2],
            fat_sector[byte_index + 3],
        ]) & 0x0FFFFFFF;

        if entry == 0 {
            free_clusters.push(cluster as u32);
            if free_clusters.len() == count {
                return Ok(free_clusters);
            }
        }
    }
    Err(FatError::NoFreeClusters)
}

/// Write cluster function
fn write_cluster<B: BlockDevice>(
    volume: &mut Fat32Volume<B>,
    cluster: u32,
    data: &[u8],
) -> Result<(), FatError> {
    let cluster_size = volume.cluster_size() as usize;
    let first_data_sector = volume.boot.reserved_sectors as u64
        + (volume.boot.fat_count as u64 * volume.boot.fat_size_sectors as u64);
    let sector_number = first_data_sector
        + ((cluster - 2) as u64 * volume.boot.sectors_per_cluster as u64);

    let mut sector_buf = vec![0u8; cluster_size];
    sector_buf[..data.len()].copy_from_slice(data);

    for i in 0..volume.boot.sectors_per_cluster {
        let offset = (i as usize) * 512;
        let sector_data = &sector_buf[offset..offset + 512];
        volume.device.write_sector(sector_number + i as u64, sector_data);
    }
    Ok(())
}

/// Update FAT entries for allocated clusters
fn update_fat_entries<B: BlockDevice>(
    volume: &mut Fat32Volume<B>,
    clusters: &[u32],
) -> Result<(), FatError> {
    let fat_start = volume.boot.reserved_sectors as u64;
    let fat_size = volume.boot.fat_size_sectors as u64;
    let mut fat_sector = [0u8; 512];

    for (i, &cluster) in clusters.iter().enumerate() {
        let next_cluster = if i == clusters.len() - 1 {
            0x0FFFFFFF // end of chain
        } else {
            clusters[i + 1]
        };

        let fat_offset = cluster as u64 * 4;
        let sector_index = fat_offset / 512;
        let byte_index = (fat_offset % 512) as usize;

        if sector_index >= fat_size {
            return Err(FatError::InvalidCluster);
        }

        volume.device.read_sector(fat_start + sector_index, &mut fat_sector);
        fat_sector[byte_index..byte_index + 4]
            .copy_from_slice(&next_cluster.to_le_bytes());
        volume.device
            .write_sector(fat_start + sector_index, &fat_sector);
    }
    Ok(())
}

/// Add directory entry for the new file
fn add_directory_entry<B: BlockDevice>(
    volume: &mut Fat32Volume<B>,
    dir_cluster: u32,
    filename: &str,
    clusters: &[u32],
    file_size: u32,
) -> Result<(), FatError> {
    // find a free slot in the directory
    // The directory can be in multiple clusters, we need to iterate through them
    let cluster_size = volume.cluster_size() as usize;
    let first_data_sector = volume.boot.reserved_sectors as u64
        + (volume.boot.fat_count as u64 * volume.boot.fat_size_sectors as u64);
    let mut current_cluster = dir_cluster;
    loop {
        let sector_number = first_data_sector
            + ((current_cluster - 2) as u64 * volume.boot.sectors_per_cluster as u64);
        let mut sector_buf = vec![0u8; cluster_size];

        for i in 0..volume.boot.sectors_per_cluster {
            volume
                .device
                .read_sector(sector_number + i as u64, &mut sector_buf[(i as usize) * 512..(i as usize + 1) * 512]);
        }

        for entry_index in 0..(cluster_size / 32) {
            let entry_offset = entry_index * 32;
            if sector_buf[entry_offset] == 0x00 || sector_buf[entry_offset] == 0xE5 {
                // free entry found
                let mut entry = [0u8; 32];
                // filename (8.3 format)
                let (name, ext) = if let Some(dot_pos) = filename.find('.') {
                    (&filename[..dot_pos], &filename[dot_pos + 1..])
                } else {
                    (filename, "")
                };
                let name_bytes = name.as_bytes();
                let ext_bytes = ext.as_bytes();
                for i in 0..8 {
                    entry[i] = if i < name_bytes.len() { name_bytes[i] } else { b' ' };
                }
                for i in 0..3 {
                    entry[8 + i] = if i < ext_bytes.len() { ext_bytes[i] } else { b' ' };
                }
                entry[11] = ATTR_ARCHIVE; // file attribute
                // first cluster
                let first_cluster = clusters[0];
                entry[20..22].copy_from_slice(&(first_cluster as u16).to_le_bytes());
                entry[26..28].copy_from_slice(&((first_cluster >> 16) as u16).to_le_bytes());
                // file size
                entry[28..32].copy_from_slice(&file_size.to_le_bytes());

                // write entry back to sector buffer
                sector_buf[entry_offset..entry_offset + 32].copy_from_slice(&entry);

                // write back modified sectors
                for i in 0..volume.boot.sectors_per_cluster {
                    volume.device.write_sector(
                        sector_number + i as u64,
                        &sector_buf[(i as usize) * 512..(i as usize + 1) * 512],
                    );
                }
                return Ok(());
            }
        }
        // move to next cluster in directory
        let next_cluster = get_next_cluster(volume, current_cluster)?;
        if next_cluster >= 0x0FFFFFF8 {
            break; // end of directory
        }
        current_cluster = next_cluster;
    }
    Err(FatError::NoFreeDirectoryEntry)
}

/// Get next cluster in FAT chain
fn get_next_cluster<B: BlockDevice>(
    volume: &Fat32Volume<B>,
    cluster: u32,
) -> Result<u32, FatError> {
    let fat_start = volume.boot.reserved_sectors as u64;
    let fat_size = volume.boot.fat_size_sectors as u64;
    let mut fat_sector = [0u8; 512];
    let fat_offset = cluster as u64 * 4;
    let sector_index = fat_offset / 512;
    let byte_index = (fat_offset % 512) as usize;
    if sector_index >= fat_size {
        return Err(FatError::InvalidCluster);
    }
    volume.device.read_sector(fat_start + sector_index, &mut fat_sector);
    let entry = u32::from_le_bytes([
        fat_sector[byte_index],
        fat_sector[byte_index + 1],
        fat_sector[byte_index + 2],
        fat_sector[byte_index + 3],
    ]) & 0x0FFFFFFF;
    Ok(entry)
}