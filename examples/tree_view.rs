// (C) 2025 - Enzo Lombardi
// Outline Demo - demonstrates hierarchical tree view

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::event::KB_CTRL_C;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::View;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::outline::{Node, OutlineViewer};
use turbo_vision::views::static_text::StaticTextBuilder;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut dialog = DialogBuilder::new().bounds(Rect::new(10, 2, 78, 22)).title("File System Tree Demo").build();

    // Add instructions
    dialog.add(Box::new(
        StaticTextBuilder::new()
            .bounds(Rect::new(2, 2, 64, 4))
            .text("Use arrows to navigate, Enter to toggle, → expand, ← collapse\nCTRL+C to exit.")
            .build(),
    ));

    // Create a sample file system tree
    let root = create_file_tree();

    // Create outline viewer
    let mut outline = OutlineViewer::new(Rect::new(2, 5, 64, 17), |name: &String| name.clone());
    outline.add_root(root);

    dialog.add(Box::new(outline));
    app.desktop.add(Box::new(dialog));

    // Simple event loop
    loop {
        app.desktop.draw(&mut app.terminal);
        let _ = app.terminal.flush();

        if let Some(mut event) = app.terminal.poll_event(Duration::from_millis(50)).ok().flatten() {
            app.desktop.handle_event(&mut event);

            // Check for quit command
            if event.what == turbo_vision::core::event::EventType::Command && event.command == turbo_vision::core::command::CM_QUIT {
                break;
            }

            // Handle Ctrl+C or F10
            if event.what == turbo_vision::core::event::EventType::Keyboard {
                let key = event.key_code;
                if key == KB_CTRL_C || key == turbo_vision::core::event::KB_F10 {
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Create a sample file system tree
fn create_file_tree() -> Rc<RefCell<Node<String>>> {
    // Root
    let root = Rc::new(RefCell::new(Node::new("/home/user".to_string())));

    // Documents folder
    let docs = Rc::new(RefCell::new(Node::new("Documents".to_string())));
    docs.borrow_mut().add_child(Rc::new(RefCell::new(Node::new("report.pdf".to_string()))));
    docs.borrow_mut().add_child(Rc::new(RefCell::new(Node::new("notes.txt".to_string()))));

    let projects = Rc::new(RefCell::new(Node::new("projects".to_string())));
    projects.borrow_mut().add_child(Rc::new(RefCell::new(Node::new("project1.doc".to_string()))));
    projects.borrow_mut().add_child(Rc::new(RefCell::new(Node::new("project2.doc".to_string()))));
    docs.borrow_mut().add_child(projects);

    root.borrow_mut().add_child(docs);

    // Code folder
    let code = Rc::new(RefCell::new(Node::new("Code".to_string())));

    let rust = Rc::new(RefCell::new(Node::new("rust".to_string())));
    rust.borrow_mut().add_child(Rc::new(RefCell::new(Node::new("Cargo.toml".to_string()))));
    rust.borrow_mut().add_child(Rc::new(RefCell::new(Node::new("main.rs".to_string()))));
    rust.borrow_mut().add_child(Rc::new(RefCell::new(Node::new("lib.rs".to_string()))));
    code.borrow_mut().add_child(rust);

    let python = Rc::new(RefCell::new(Node::new("python".to_string())));
    python.borrow_mut().add_child(Rc::new(RefCell::new(Node::new("app.py".to_string()))));
    python.borrow_mut().add_child(Rc::new(RefCell::new(Node::new("utils.py".to_string()))));
    code.borrow_mut().add_child(python);

    root.borrow_mut().add_child(code);

    // Downloads folder
    let downloads = Rc::new(RefCell::new(Node::new("Downloads".to_string())));
    downloads.borrow_mut().add_child(Rc::new(RefCell::new(Node::new("file1.zip".to_string()))));
    downloads.borrow_mut().add_child(Rc::new(RefCell::new(Node::new("file2.tar.gz".to_string()))));
    downloads.borrow_mut().add_child(Rc::new(RefCell::new(Node::new("image.png".to_string()))));

    root.borrow_mut().add_child(downloads);

    // Config files
    root.borrow_mut().add_child(Rc::new(RefCell::new(Node::new(".bashrc".to_string()))));
    root.borrow_mut().add_child(Rc::new(RefCell::new(Node::new(".vimrc".to_string()))));

    root
}
