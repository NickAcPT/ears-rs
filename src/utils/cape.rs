use image::{RgbaImage, imageops::{crop_imm, self}};

pub fn process_ears_cape(ears_cape: RgbaImage) -> RgbaImage {
    if (ears_cape.width() == 64) && (ears_cape.height() == 32) {
        return ears_cape;
    }
    
    let mut final_cape = RgbaImage::new(64, 32);
    
    let front_view = crop_imm(&ears_cape, 0, 0, 20, 16);
    let back_view = crop_imm(&ears_cape, 10, 0, 20, 16);
    let left_view = crop_imm(&ears_cape, 0, 0, 1, 16);
    let right_view = crop_imm(&ears_cape, 9, 0, 1, 16);
    let top = crop_imm(&ears_cape, 0, 0, 10, 1);
    let bottom = crop_imm(&ears_cape, 10, 15, 10, 1);
    
    imageops::replace(&mut final_cape, &*front_view, 1, 1);
    imageops::replace(&mut final_cape, &*back_view, 12, 1);
    
    imageops::replace(&mut final_cape, &*left_view, 0, 1);
    imageops::replace(&mut final_cape, &*right_view, 11, 1);
    
    imageops::replace(&mut final_cape, &*top, 1, 0);
    imageops::replace(&mut final_cape, &*bottom, 12, 0);
    
    final_cape
}