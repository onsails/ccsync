//! Integration tests for file comparison

use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

use super::{ComparisonResult, ConflictStrategy, FileComparator};

#[test]
fn test_compare_identical_files() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.txt");
    let dest = tmp.path().join("dest.txt");

    let content = "identical content\n";
    fs::write(&source, content).unwrap();
    fs::write(&dest, content).unwrap();

    let _comparator = FileComparator::new();
    let result = FileComparator::compare(&source, &dest, ConflictStrategy::Fail).unwrap();

    assert_eq!(result, ComparisonResult::Identical);
}

#[test]
fn test_compare_source_only() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.txt");
    let dest = tmp.path().join("dest.txt");

    fs::write(&source, "content").unwrap();
    // dest doesn't exist

    let _comparator = FileComparator::new();
    let result = FileComparator::compare(&source, &dest, ConflictStrategy::Fail).unwrap();

    assert_eq!(result, ComparisonResult::SourceOnly);
}

#[test]
fn test_compare_destination_only() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.txt");
    let dest = tmp.path().join("dest.txt");

    // source doesn't exist
    fs::write(&dest, "content").unwrap();

    let _comparator = FileComparator::new();
    let result = FileComparator::compare(&source, &dest, ConflictStrategy::Fail).unwrap();

    assert_eq!(result, ComparisonResult::DestinationOnly);
}

#[test]
fn test_compare_conflict_source_newer() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.txt");
    let dest = tmp.path().join("dest.txt");

    // Create dest first (older)
    fs::write(&dest, "old content").unwrap();

    // Wait to ensure timestamp difference
    thread::sleep(Duration::from_millis(10));

    // Create source (newer)
    fs::write(&source, "new content").unwrap();

    let _comparator = FileComparator::new();
    let result = FileComparator::compare(&source, &dest, ConflictStrategy::Newer).unwrap();

    match result {
        ComparisonResult::Conflict {
            source_newer,
            strategy,
        } => {
            assert!(source_newer, "Source should be newer");
            assert_eq!(strategy, ConflictStrategy::Newer);
        }
        _ => panic!("Expected Conflict, got {:?}", result),
    }
}

#[test]
fn test_compare_conflict_dest_newer() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.txt");
    let dest = tmp.path().join("dest.txt");

    // Create source first (older)
    fs::write(&source, "old content").unwrap();

    // Wait to ensure timestamp difference
    thread::sleep(Duration::from_millis(10));

    // Create dest (newer)
    fs::write(&dest, "new content").unwrap();

    let _comparator = FileComparator::new();
    let result = FileComparator::compare(&source, &dest, ConflictStrategy::Newer).unwrap();

    match result {
        ComparisonResult::Conflict {
            source_newer,
            strategy,
        } => {
            assert!(!source_newer, "Destination should be newer");
            assert_eq!(strategy, ConflictStrategy::Newer);
        }
        _ => panic!("Expected Conflict, got {:?}", result),
    }
}

#[test]
fn test_diff_generation_with_changes() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.txt");
    let dest = tmp.path().join("dest.txt");

    fs::write(&dest, "line 1\nline 2\nline 3\n").unwrap();
    fs::write(&source, "line 1\nmodified line 2\nline 3\n").unwrap();

    let _comparator = FileComparator::new();
    let diff = FileComparator::generate_diff(&source, &dest).unwrap();

    // Diff should contain the file paths
    assert!(diff.contains(&source.display().to_string()));
    assert!(diff.contains(&dest.display().to_string()));

    // Should contain ANSI color codes
    assert!(diff.contains("\x1b["));
}

#[test]
fn test_all_conflict_strategies() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.txt");
    let dest = tmp.path().join("dest.txt");

    fs::write(&source, "source content").unwrap();
    fs::write(&dest, "dest content").unwrap();

    let _comparator = FileComparator::new();

    for strategy in [
        ConflictStrategy::Fail,
        ConflictStrategy::Overwrite,
        ConflictStrategy::Skip,
        ConflictStrategy::Newer,
    ] {
        let result = FileComparator::compare(&source, &dest, strategy).unwrap();

        match result {
            ComparisonResult::Conflict {
                strategy: returned_strategy,
                ..
            } => {
                assert_eq!(returned_strategy, strategy);
            }
            _ => panic!("Expected Conflict for strategy {:?}", strategy),
        }
    }
}

#[test]
fn test_multiple_comparisons() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.txt");
    let dest = tmp.path().join("dest.txt");

    fs::write(&source, "content").unwrap();
    fs::write(&dest, "different").unwrap();

    let _comparator = FileComparator::new();

    // First comparison
    let result1 = FileComparator::compare(&source, &dest, ConflictStrategy::Fail).unwrap();

    // Second comparison should produce same result
    let result2 = FileComparator::compare(&source, &dest, ConflictStrategy::Fail).unwrap();

    assert_eq!(result1, result2);
}
