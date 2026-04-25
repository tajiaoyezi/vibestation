//! MVP-05 Phase B Step 2 · Pane layout pure function 性能基线。
//!
//! 这些 bench 验证 `crates::core::panes` 的 4 个 pure function 在 MVP 上限（4 Pane）规模下
//! 是 sub-millisecond 量级，避免 IPC layer 调用层的额外噪声 · 为后续 P99 性能预算（spec §F.4
//! 分屏快捷键 < 150ms · §F.5 关 Pane < 100ms · §F.6 Smart Layout < 200ms）提供下限信心。

use criterion::{criterion_group, criterion_main, Criterion};
use vibestation_core::panes::{
    apply_smart_layout, close_pane_in_layout, split_layout, update_split_ratio, LayoutNode,
    SmartLayoutKind, SplitDir,
};

fn solo_layout(pane_id: &str) -> LayoutNode {
    LayoutNode::Single {
        pane_id: pane_id.to_string(),
    }
}

fn horizontal_2pane(left: &str, right: &str) -> LayoutNode {
    LayoutNode::Split {
        direction: SplitDir::Horizontal,
        ratio: 0.5,
        first: Box::new(solo_layout(left)),
        second: Box::new(solo_layout(right)),
    }
}

fn vertical_2pane(top: &str, bottom: &str) -> LayoutNode {
    LayoutNode::Split {
        direction: SplitDir::Vertical,
        ratio: 0.5,
        first: Box::new(solo_layout(top)),
        second: Box::new(solo_layout(bottom)),
    }
}

fn layout_2x2() -> LayoutNode {
    LayoutNode::Split {
        direction: SplitDir::Horizontal,
        ratio: 0.5,
        first: Box::new(vertical_2pane("p1", "p2")),
        second: Box::new(vertical_2pane("p3", "p4")),
    }
}

fn bench_split_layout(c: &mut Criterion) {
    c.bench_function("split_layout · solo→horizontal_2pane", |b| {
        let layout = solo_layout("p1");
        b.iter(|| {
            split_layout(
                criterion::black_box(&layout),
                criterion::black_box("p1"),
                criterion::black_box(SplitDir::Horizontal),
                criterion::black_box("p2".to_string()),
            )
            .unwrap()
        });
    });

    c.bench_function("split_layout · horizontal_2pane→2x2", |b| {
        let layout = horizontal_2pane("p1", "p2");
        b.iter(|| {
            // 在 right pane 上下分 → horizontal{p1, vertical{p2, p3}}
            split_layout(
                criterion::black_box(&layout),
                criterion::black_box("p2"),
                criterion::black_box(SplitDir::Vertical),
                criterion::black_box("p3".to_string()),
            )
            .unwrap()
        });
    });
}

fn bench_close_pane_in_layout(c: &mut Criterion) {
    c.bench_function("close_pane_in_layout · 2x2→3pane", |b| {
        let layout = layout_2x2();
        b.iter(|| {
            close_pane_in_layout(criterion::black_box(&layout), criterion::black_box("p4")).unwrap()
        });
    });

    c.bench_function("close_pane_in_layout · horizontal_2pane→solo", |b| {
        let layout = horizontal_2pane("p1", "p2");
        b.iter(|| {
            close_pane_in_layout(criterion::black_box(&layout), criterion::black_box("p2")).unwrap()
        });
    });
}

fn bench_update_split_ratio(c: &mut Criterion) {
    c.bench_function("update_split_ratio · 2x2 root", |b| {
        let layout = layout_2x2();
        b.iter(|| {
            update_split_ratio(
                criterion::black_box(&layout),
                criterion::black_box("p1"),
                criterion::black_box(0.7),
            )
            .unwrap()
        });
    });
}

fn bench_apply_smart_layout(c: &mut Criterion) {
    c.bench_function("apply_smart_layout · 2x2→Solo", |b| {
        let layout = layout_2x2();
        b.iter(|| {
            apply_smart_layout(
                criterion::black_box(&layout),
                criterion::black_box(SmartLayoutKind::Solo),
                criterion::black_box("p1"),
            )
            .unwrap()
        });
    });

    c.bench_function("apply_smart_layout · 2x2→AiAndRunner", |b| {
        let layout = layout_2x2();
        b.iter(|| {
            apply_smart_layout(
                criterion::black_box(&layout),
                criterion::black_box(SmartLayoutKind::AiAndRunner),
                criterion::black_box("p1"),
            )
            .unwrap()
        });
    });
}

criterion_group!(
    benches,
    bench_split_layout,
    bench_close_pane_in_layout,
    bench_update_split_ratio,
    bench_apply_smart_layout,
);
criterion_main!(benches);
