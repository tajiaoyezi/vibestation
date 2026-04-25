use crate::db;
use crate::panes::{LayoutNode, SplitDir};
use rusqlite::Connection;
use tempfile::TempDir;

pub fn create_fixture_solo_layout() -> (TempDir, Connection) {
    create_fixture(vec!["p1"], solo_layout(), "p1")
}

pub fn create_fixture_horizontal_2pane() -> (TempDir, Connection) {
    create_fixture(
        vec!["p1", "p2"],
        LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(LayoutNode::Single {
                pane_id: "p1".to_string(),
            }),
            second: Box::new(LayoutNode::Single {
                pane_id: "p2".to_string(),
            }),
        },
        "p1",
    )
}

pub fn create_fixture_vertical_2pane() -> (TempDir, Connection) {
    create_fixture(
        vec!["p1", "p2"],
        LayoutNode::Split {
            direction: SplitDir::Vertical,
            ratio: 0.5,
            first: Box::new(LayoutNode::Single {
                pane_id: "p1".to_string(),
            }),
            second: Box::new(LayoutNode::Single {
                pane_id: "p2".to_string(),
            }),
        },
        "p1",
    )
}

pub fn create_fixture_2x2_layout() -> (TempDir, Connection) {
    create_fixture(
        vec!["p1", "p2", "p3", "p4"],
        LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(LayoutNode::Split {
                direction: SplitDir::Vertical,
                ratio: 0.5,
                first: Box::new(LayoutNode::Single {
                    pane_id: "p1".to_string(),
                }),
                second: Box::new(LayoutNode::Single {
                    pane_id: "p2".to_string(),
                }),
            }),
            second: Box::new(LayoutNode::Split {
                direction: SplitDir::Vertical,
                ratio: 0.5,
                first: Box::new(LayoutNode::Single {
                    pane_id: "p3".to_string(),
                }),
                second: Box::new(LayoutNode::Single {
                    pane_id: "p4".to_string(),
                }),
            }),
        },
        "p1",
    )
}

pub fn create_fixture_invalid_3horizontal() -> Vec<LayoutNode> {
    vec![
        LayoutNode::Single {
            pane_id: "p1".to_string(),
        },
        LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(LayoutNode::Single {
                pane_id: "p2".to_string(),
            }),
            second: Box::new(LayoutNode::Single {
                pane_id: "p3".to_string(),
            }),
        },
    ]
}

pub fn create_fixture_invalid_3vertical() -> Vec<LayoutNode> {
    vec![
        LayoutNode::Single {
            pane_id: "p1".to_string(),
        },
        LayoutNode::Split {
            direction: SplitDir::Vertical,
            ratio: 0.5,
            first: Box::new(LayoutNode::Single {
                pane_id: "p2".to_string(),
            }),
            second: Box::new(LayoutNode::Single {
                pane_id: "p3".to_string(),
            }),
        },
    ]
}

fn create_fixture(
    pane_ids: Vec<&str>,
    layout: LayoutNode,
    focused_pane_id: &str,
) -> (TempDir, Connection) {
    let dir = tempfile::tempdir().unwrap();
    let conn = Connection::open_in_memory().unwrap();
    db::migrate(&conn).unwrap();

    conn.execute(
        "INSERT INTO workspaces (workspace_id, name, path, has_git, repo_root, created_at, last_opened)
         VALUES ('w1', 'Fixture Workspace', ?1, 0, NULL, 1, 1)",
        [dir.path().to_string_lossy().to_string()],
    )
    .unwrap();

    let layout_json = serde_json::to_string(&layout).unwrap();
    conn.execute(
        "INSERT INTO tabs (tab_id, workspace_id, name, shell, cwd, scroll_back, created_at, layout, focused_pane_id)
         VALUES ('t1', 'w1', 'Fixture Tab', '/bin/zsh', '/tmp', '[]', 1, ?1, ?2)",
        rusqlite::params![layout_json, focused_pane_id],
    )
    .unwrap();

    for (index, pane_id) in pane_ids.iter().enumerate() {
        conn.execute(
            "INSERT INTO panes (pane_id, tab_id, shell, cwd, scroll_back, created_at)
             VALUES (?1, 't1', '/bin/zsh', '/tmp', '[]', ?2)",
            rusqlite::params![pane_id, index as i64 + 1],
        )
        .unwrap();
    }

    (dir, conn)
}

fn solo_layout() -> LayoutNode {
    LayoutNode::Single {
        pane_id: "p1".to_string(),
    }
}
