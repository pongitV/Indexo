pub mod mover;

#[cfg(test)]
mod tests {
    use super::mover::*;
    use std::fs;

    #[test]
    fn test_safe_move_and_undo() {
        let temp_dir = std::env::temp_dir().join(format!("organizador_test_{}", uuid::Uuid::new_v4()));
        let src_dir = temp_dir.join("origem");
        let dest_dir = temp_dir.join("destino");
        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&dest_dir).unwrap();

        let src_file = src_dir.join("documento.txt");
        fs::write(&src_file, "conteudo de teste").unwrap();

        let dest_file = dest_dir.join("documento.txt");
        let final_dest = safe_move(&src_file, &dest_file).unwrap();

        assert!(!src_file.exists());
        assert!(final_dest.exists());
        assert_eq!(fs::read_to_string(&final_dest).unwrap(), "conteudo de teste");

        // Testar colisão (segundo move para mesmo nome não sobrescreve)
        let src_file_2 = src_dir.join("documento.txt");
        fs::write(&src_file_2, "conteudo novo").unwrap();
        let final_dest_2 = safe_move(&src_file_2, &dest_file).unwrap();

        assert!(final_dest_2.to_string_lossy().contains("(1)"));
        assert!(final_dest.exists());
        assert!(final_dest_2.exists());
        assert_eq!(fs::read_to_string(&final_dest_2).unwrap(), "conteudo novo");

        // Testar undo
        let restored = undo_single_move(&src_file, &final_dest).unwrap();
        assert!(restored.exists());

        // Limpeza
        let _ = fs::remove_dir_all(temp_dir);
    }
}
