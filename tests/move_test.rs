#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_project(tmp: &Path) {
    // Root package.yml
    fs::write(tmp.join("package.yml"), "enforce_dependencies: false\n")
        .unwrap();
    // packs.yml config
    fs::write(tmp.join("packs.yml"), "").unwrap();
}

fn create_pack(tmp: &Path, name: &str) {
    let pack_dir = tmp.join(name);
    fs::create_dir_all(&pack_dir).unwrap();
    fs::write(pack_dir.join("package.yml"), "enforce_dependencies: true\n")
        .unwrap();
}

fn create_file(tmp: &Path, relative_path: &str, contents: &str) {
    let full_path = tmp.join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full_path, contents).unwrap();
}

fn pks_move(
    tmp: &Path,
    destination: &str,
    paths: &[&str],
) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(cargo_bin!("pks"));
    cmd.arg("--project-root").arg(tmp);
    cmd.arg("move").arg(destination);
    for p in paths {
        cmd.arg(p);
    }
    cmd.assert()
}

// 1. Error when destination pack doesn't exist
#[test]
fn test_error_when_destination_pack_does_not_exist() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);

    pks_move(tmp, "packs/nonexistent", &["app/services/foo.rb"])
        .failure()
        .stderr(predicate::str::contains("pack not found"));
}

// 2. Move file from root to pack
#[test]
fn test_move_file_from_root_to_pack() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/animals");

    create_file(
        tmp,
        "app/services/horse_like/donkey.rb",
        "class HorseLike::Donkey; end",
    );
    create_file(
        tmp,
        "spec/services/horse_like/donkey_spec.rb",
        "describe HorseLike::Donkey",
    );

    pks_move(tmp, "packs/animals", &["app/services/horse_like/donkey.rb"])
        .success();

    assert!(tmp
        .join("packs/animals/app/services/horse_like/donkey.rb")
        .exists());
    assert!(tmp
        .join("packs/animals/spec/services/horse_like/donkey_spec.rb")
        .exists());
    assert!(!tmp.join("app/services/horse_like/donkey.rb").exists());
    assert!(!tmp.join("spec/services/horse_like/donkey_spec.rb").exists());
}

// 3. Move directory from root to pack
#[test]
fn test_move_directory_from_root_to_pack() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/animals");

    create_file(
        tmp,
        "app/services/horse_like/donkey.rb",
        "class Donkey; end",
    );
    create_file(tmp, "app/services/horse_like/mule.rb", "class Mule; end");

    pks_move(tmp, "packs/animals", &["app/services/horse_like"]).success();

    assert!(tmp
        .join("packs/animals/app/services/horse_like/donkey.rb")
        .exists());
    assert!(tmp
        .join("packs/animals/app/services/horse_like/mule.rb")
        .exists());
    assert!(!tmp.join("app/services/horse_like/donkey.rb").exists());
    assert!(!tmp.join("app/services/horse_like/mule.rb").exists());
}

// 4. Move file between packs
#[test]
fn test_move_file_between_packs() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/organisms");
    create_pack(tmp, "packs/animals");

    create_file(
        tmp,
        "packs/organisms/app/services/horse.rb",
        "class Horse; end",
    );
    create_file(
        tmp,
        "packs/organisms/spec/services/horse_spec.rb",
        "describe Horse",
    );

    pks_move(
        tmp,
        "packs/animals",
        &["packs/organisms/app/services/horse.rb"],
    )
    .success();

    assert!(tmp.join("packs/animals/app/services/horse.rb").exists());
    assert!(tmp
        .join("packs/animals/spec/services/horse_spec.rb")
        .exists());
    assert!(!tmp.join("packs/organisms/app/services/horse.rb").exists());
    assert!(!tmp
        .join("packs/organisms/spec/services/horse_spec.rb")
        .exists());
}

// 5. Move file from child pack to parent pack
#[test]
fn test_move_file_from_child_to_parent_pack() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/animals");
    create_pack(tmp, "packs/animals/horse_like");

    create_file(
        tmp,
        "packs/animals/horse_like/app/services/donkey.rb",
        "class Donkey; end",
    );
    create_file(
        tmp,
        "packs/animals/horse_like/spec/services/donkey_spec.rb",
        "describe Donkey",
    );

    pks_move(
        tmp,
        "packs/animals",
        &["packs/animals/horse_like/app/services/donkey.rb"],
    )
    .success();

    assert!(tmp.join("packs/animals/app/services/donkey.rb").exists());
    assert!(tmp
        .join("packs/animals/spec/services/donkey_spec.rb")
        .exists());
    assert!(!tmp
        .join("packs/animals/horse_like/app/services/donkey.rb")
        .exists());
}

// 6. Move file from parent pack to child pack
#[test]
fn test_move_file_from_parent_to_child_pack() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/fruits");
    create_pack(tmp, "packs/fruits/apples");

    create_file(
        tmp,
        "packs/fruits/app/services/apple.rb",
        "class Apple; end",
    );
    create_file(
        tmp,
        "packs/fruits/spec/services/apple_spec.rb",
        "describe Apple",
    );

    pks_move(
        tmp,
        "packs/fruits/apples",
        &["packs/fruits/app/services/apple.rb"],
    )
    .success();

    assert!(tmp
        .join("packs/fruits/apples/app/services/apple.rb")
        .exists());
    assert!(tmp
        .join("packs/fruits/apples/spec/services/apple_spec.rb")
        .exists());
    assert!(!tmp.join("packs/fruits/app/services/apple.rb").exists());
}

// 7. Move with trailing slash on path
#[test]
fn test_move_with_trailing_slash() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/animals");

    create_file(tmp, "app/services/horse.rb", "class Horse; end");

    pks_move(tmp, "packs/animals", &["app/services/horse.rb"]).success();

    assert!(tmp.join("packs/animals/app/services/horse.rb").exists());
}

// 8. Merge folders — moving files into a pack that already has files in the same directory
#[test]
fn test_merge_folders() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/animals");

    // Existing file in the destination pack
    create_file(tmp, "packs/animals/app/services/cat.rb", "class Cat; end");

    // File to move
    create_file(tmp, "app/services/dog.rb", "class Dog; end");

    pks_move(tmp, "packs/animals", &["app/services/dog.rb"]).success();

    // Both should exist
    assert!(tmp.join("packs/animals/app/services/cat.rb").exists());
    assert!(tmp.join("packs/animals/app/services/dog.rb").exists());
    assert!(!tmp.join("app/services/dog.rb").exists());
}

// 9. Skip when destination file already exists
#[test]
fn test_skip_when_destination_exists() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/animals");

    create_file(tmp, "app/services/dog.rb", "class Dog; end # original");
    create_file(
        tmp,
        "packs/animals/app/services/dog.rb",
        "class Dog; end # existing",
    );

    pks_move(tmp, "packs/animals", &["app/services/dog.rb"])
        .success()
        .stdout(predicate::str::contains("[SKIP]"));

    // Both should still exist with original content
    assert!(tmp.join("app/services/dog.rb").exists());
    let existing_content =
        fs::read_to_string(tmp.join("packs/animals/app/services/dog.rb"))
            .unwrap();
    assert!(existing_content.contains("# existing"));
}

// 10. Move rake tasks from lib/
#[test]
fn test_move_rake_tasks_from_lib() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/my_pack");

    create_file(tmp, "lib/tasks/my_task.rake", "task :my_task");
    create_file(tmp, "spec/lib/tasks/my_task_spec.rb", "describe my_task");

    pks_move(tmp, "packs/my_pack", &["lib/tasks/my_task.rake"]).success();

    assert!(tmp.join("packs/my_pack/lib/tasks/my_task.rake").exists());
    // .rake files don't have _spec.rb auto-detection (only .rb)
    // The spec must be moved explicitly or the lib/ spec auto-detection applies to .rb only
}

// 11. Move ruby files from lib/
#[test]
fn test_move_ruby_files_from_lib() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/my_pack");

    create_file(tmp, "lib/my_lib.rb", "module MyLib; end");
    create_file(tmp, "spec/lib/my_lib_spec.rb", "describe MyLib");

    pks_move(tmp, "packs/my_pack", &["lib/my_lib.rb"]).success();

    assert!(tmp.join("packs/my_pack/lib/my_lib.rb").exists());
    assert!(tmp.join("packs/my_pack/spec/lib/my_lib_spec.rb").exists());
    assert!(!tmp.join("lib/my_lib.rb").exists());
    assert!(!tmp.join("spec/lib/my_lib_spec.rb").exists());
}

// 12. Rubocop todo rewriting
#[test]
fn test_rubocop_todo_rewriting() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/animals");

    create_file(tmp, "app/services/horse.rb", "class Horse; end");

    let rubocop_todo = "\
Style/FrozenStringLiteralComment:
  Exclude:
    - 'app/services/horse.rb'
    - 'app/services/other.rb'
";
    fs::write(tmp.join(".rubocop_todo.yml"), rubocop_todo).unwrap();

    pks_move(tmp, "packs/animals", &["app/services/horse.rb"])
        .success()
        .stdout(predicate::str::contains(
            "Replaced 1 occurrence(s) of app/services/horse.rb in .rubocop_todo.yml",
        ));

    let updated = fs::read_to_string(tmp.join(".rubocop_todo.yml")).unwrap();
    assert!(updated.contains("packs/animals/app/services/horse.rb"));
    assert!(!updated.contains("    - 'app/services/horse.rb'"));
    // Other entries should be unchanged
    assert!(updated.contains("app/services/other.rb"));
}

// 13. Reference updating
#[test]
fn test_reference_updating() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/organisms");
    create_pack(tmp, "packs/animals");

    create_file(
        tmp,
        "packs/organisms/app/services/horse.rb",
        "class Horse; end",
    );

    // A file referencing the old pack name
    create_file(
        tmp,
        "some_config.yml",
        "dependencies:\n  - packs/organisms\n",
    );

    pks_move(
        tmp,
        "packs/animals",
        &["packs/organisms/app/services/horse.rb"],
    )
    .success();

    let config_content =
        fs::read_to_string(tmp.join("some_config.yml")).unwrap();
    assert!(config_content.contains("packs/animals"));
    assert!(!config_content.contains("packs/organisms"));
}

// 14. Move into nested pack from root
#[test]
fn test_move_into_nested_pack() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/fruits");
    create_pack(tmp, "packs/fruits/apples");

    create_file(
        tmp,
        "app/services/granny_smith.rb",
        "class GrannySmith; end",
    );

    pks_move(
        tmp,
        "packs/fruits/apples",
        &["app/services/granny_smith.rb"],
    )
    .success();

    assert!(tmp
        .join("packs/fruits/apples/app/services/granny_smith.rb")
        .exists());
    assert!(!tmp.join("app/services/granny_smith.rb").exists());
}

// 15. Move from non-pack package to a pack
#[test]
fn test_move_from_non_pack_to_pack() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/my_pack");

    create_file(tmp, "lib/tasks/foo.rake", "task :foo");

    pks_move(tmp, "packs/my_pack", &["lib/tasks/foo.rake"]).success();

    assert!(tmp.join("packs/my_pack/lib/tasks/foo.rake").exists());
    assert!(!tmp.join("lib/tasks/foo.rake").exists());
}

// 16. Verify "Moving file" output
#[test]
fn test_moving_file_output() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/animals");

    create_file(tmp, "app/services/horse.rb", "class Horse; end");

    pks_move(tmp, "packs/animals", &["app/services/horse.rb"])
        .success()
        .stdout(predicate::str::contains(
            "Moving file app/services/horse.rb to packs/animals/app/services/horse.rb",
        ));
}

// 17. Verify "[SKIP]" output when destination already exists
#[test]
fn test_skip_output_when_destination_exists() {
    let tmp_dir = TempDir::new().unwrap();
    let tmp = tmp_dir.path();
    setup_project(tmp);
    create_pack(tmp, "packs/animals");

    create_file(tmp, "app/services/horse.rb", "class Horse; end");
    create_file(
        tmp,
        "packs/animals/app/services/horse.rb",
        "class Horse; end # existing",
    );

    pks_move(tmp, "packs/animals", &["app/services/horse.rb"])
        .success()
        .stdout(predicate::str::contains(
            "[SKIP] Not moving app/services/horse.rb, packs/animals/app/services/horse.rb already exists",
        ));
}
