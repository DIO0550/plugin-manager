use super::mock::MockFs;
use super::*;

#[test]
fn test_mock_fs_file_operations() {
    let fs = MockFs::new();

    // ファイル追加
    fs.add_file("/test.txt", "hello");
    assert!(fs.exists(Path::new("/test.txt")));
    assert!(!fs.is_dir(Path::new("/test.txt")));

    // 内容読み込み
    let content = fs.read_to_string(Path::new("/test.txt")).unwrap();
    assert_eq!(content, "hello");

    // コピー
    fs.copy_file(Path::new("/test.txt"), Path::new("/copy.txt"))
        .unwrap();
    assert!(fs.exists(Path::new("/copy.txt")));

    // 削除
    fs.remove(Path::new("/test.txt")).unwrap();
    assert!(!fs.exists(Path::new("/test.txt")));
}

#[test]
fn test_mock_fs_content_hash() {
    let fs = MockFs::new();

    fs.add_file("/a.txt", "same content");
    fs.add_file("/b.txt", "same content");
    fs.add_file("/c.txt", "different");

    let hash_a = fs.content_hash(Path::new("/a.txt")).unwrap();
    let hash_b = fs.content_hash(Path::new("/b.txt")).unwrap();
    let hash_c = fs.content_hash(Path::new("/c.txt")).unwrap();

    assert_eq!(hash_a, hash_b);
    assert_ne!(hash_a, hash_c);
}
