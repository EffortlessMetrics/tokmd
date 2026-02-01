//! Tests for render_obj and render_midi to kill arithmetic mutation survivors.
//!
//! These tests verify the arithmetic operations in the fun rendering functions:
//! - render_obj: grid positioning (idx % 5, idx / 5) and height calculation
//! - render_midi: key (60 + depth % 12), velocity (40 + lines/2), start time (idx * 240)
#![cfg(feature = "fun")]

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::{
    ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, BoilerplateReport,
    DerivedReport, DerivedTotals, DistributionReport, FileStatRow, HistogramBucket,
    IntegrityReport, LangPurityReport, MaxFileReport, NestingReport, PolyglotReport, RateReport,
    RateRow, RatioReport, RatioRow, ReadingTimeReport, TestDensityReport, TopOffenders,
};
use tokmd_config::AnalysisFormat;
use tokmd_types::{ScanStatus, ToolInfo};

/// Create a minimal DerivedReport with the given largest_lines rows.
fn make_derived_report(largest_lines: Vec<FileStatRow>) -> DerivedReport {
    DerivedReport {
        totals: DerivedTotals {
            files: largest_lines.len(),
            code: 100,
            comments: 10,
            blanks: 5,
            lines: 115,
            bytes: 1000,
            tokens: 250,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 10,
                denominator: 100,
                ratio: 0.1,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 5,
                denominator: 115,
                ratio: 0.043,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 1000,
                denominator: 115,
                rate: 8.7,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/lib.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 100,
                comments: 10,
                blanks: 5,
                lines: 115,
                bytes: 1000,
                tokens: 250,
                doc_pct: Some(0.1),
                bytes_per_line: Some(8.7),
                depth: 1,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 1,
            avg: 1.0,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 0,
            prod_lines: 100,
            test_files: 0,
            prod_files: 1,
            ratio: 0.0,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 0,
            logic_lines: 100,
            ratio: 0.0,
            infra_langs: vec![],
        },
        polyglot: PolyglotReport {
            lang_count: 1,
            entropy: 0.0,
            dominant_lang: "Rust".into(),
            dominant_lines: 100,
            dominant_pct: 1.0,
        },
        distribution: DistributionReport {
            count: 1,
            min: 115,
            max: 115,
            mean: 115.0,
            median: 115.0,
            p90: 115.0,
            p99: 115.0,
            gini: 0.0,
        },
        histogram: vec![HistogramBucket {
            label: "0-100".into(),
            min: 0,
            max: Some(100),
            files: 1,
            pct: 1.0,
        }],
        top: TopOffenders {
            largest_lines,
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 1.0,
            lines_per_minute: 115,
            basis_lines: 115,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "0".repeat(64),
            entries: 1,
        },
    }
}

/// Create a test AnalysisReceipt with the given derived report.
fn make_receipt(derived: DerivedReport) -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "test".into(),
        },
        mode: "analyze".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: AnalysisSource {
            inputs: vec![".".into()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: "separate".into(),
        },
        args: AnalysisArgsMeta {
            preset: "receipt".into(),
            format: "obj".into(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
            import_granularity: "module".into(),
        },
        archetype: None,
        topics: None,
        entropy: None,
        predictive_churn: None,
        corporate_fingerprint: None,
        license: None,
        derived: Some(derived),
        assets: None,
        deps: None,
        git: None,
        imports: None,
        dup: None,
        complexity: None,
        fun: None,
    }
}

/// Create a FileStatRow with specific depth and lines values for testing.
fn make_file_row(path: &str, depth: usize, lines: usize) -> FileStatRow {
    FileStatRow {
        path: path.into(),
        module: "src".into(),
        lang: "Rust".into(),
        code: lines.saturating_sub(10),
        comments: 5,
        blanks: 5,
        lines,
        bytes: lines * 10,
        tokens: lines * 2,
        doc_pct: Some(0.05),
        bytes_per_line: Some(10.0),
        depth,
    }
}

// =============================================================================
// render_obj tests
// =============================================================================

#[test]
fn render_obj_grid_positioning() {
    // Create 7 rows to exercise indices 0-6, testing idx % 5 and idx / 5
    // Key test values:
    // - idx=4: 4 % 5 = 4, 4 / 5 = 0  (x=8.0, y=0.0)
    // - idx=5: 5 % 5 = 0, 5 / 5 = 1  (x=0.0, y=2.0) - wrap test
    // - idx=6: 6 % 5 = 1, 6 / 5 = 1  (x=2.0, y=2.0)
    let rows: Vec<FileStatRow> = (0..7)
        .map(|i| make_file_row(&format!("file{}.rs", i), 1, 100))
        .collect();

    let derived = make_derived_report(rows);
    let receipt = make_receipt(derived);

    let output = render(&receipt, AnalysisFormat::Obj).expect("render OBJ");
    let obj_text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text output for OBJ"),
    };

    // Parse OBJ vertices to verify grid positioning
    // Each building has 8 vertices. With 7 buildings, we have 56 vertices.
    // OBJ vertex generation: building at (x, y) with w=1.5, d=1.5 creates vertices:
    //   (x, y), (x+w, y), (x+w, y+d), (x, y+d) for bottom face
    //   Same for top face at z+h
    // So x ranges from x to x+1.5, y ranges from y to y+1.5

    let vertices: Vec<(f32, f32, f32)> = obj_text
        .lines()
        .filter(|l| l.starts_with("v "))
        .map(|l| {
            let parts: Vec<f32> = l[2..]
                .split_whitespace()
                .map(|p| p.parse().unwrap())
                .collect();
            (parts[0], parts[1], parts[2])
        })
        .collect();

    // Verify we have vertices for 7 buildings (8 vertices each)
    assert_eq!(vertices.len(), 56, "expected 56 vertices for 7 buildings");

    // Check that idx=5 wraps correctly (x should start at 0, not 10)
    // Building 5: x = (5 % 5) * 2.0 = 0.0, so vertices at x=0 and x=1.5
    let building_5_start = 5 * 8;
    let b5_vertices = &vertices[building_5_start..building_5_start + 8];
    let b5_x_values: Vec<f32> = b5_vertices.iter().map(|(x, _, _)| *x).collect();

    assert!(
        b5_x_values.iter().any(|&x| x.abs() < 0.01),
        "building 5 should have x=0 vertex, got {:?}",
        b5_x_values
    );
    assert!(
        b5_x_values.iter().any(|&x| (x - 1.5).abs() < 0.01),
        "building 5 should have x=1.5 vertex, got {:?}",
        b5_x_values
    );

    // Building 5's y should be at row 1: (5 / 5) * 2.0 = 2.0
    // So vertices at y=2.0 and y=3.5 (2.0 + 1.5)
    let b5_y_values: Vec<f32> = b5_vertices.iter().map(|(_, y, _)| *y).collect();
    assert!(
        b5_y_values.iter().any(|&y| (y - 2.0).abs() < 0.01),
        "building 5 should have y=2.0 vertex, got {:?}",
        b5_y_values
    );
    assert!(
        b5_y_values.iter().any(|&y| (y - 3.5).abs() < 0.01),
        "building 5 should have y=3.5 vertex, got {:?}",
        b5_y_values
    );

    // idx=4: (4 % 5) * 2.0 = 8.0, (4 / 5) * 2.0 = 0.0
    // Vertices at x=8.0 and x=9.5
    let building_4_start = 4 * 8;
    let b4_vertices = &vertices[building_4_start..building_4_start + 8];
    let b4_x_values: Vec<f32> = b4_vertices.iter().map(|(x, _, _)| *x).collect();
    assert!(
        b4_x_values.iter().any(|&x| (x - 8.0).abs() < 0.01),
        "building 4 should have x=8.0 vertex, got {:?}",
        b4_x_values
    );
    assert!(
        b4_x_values.iter().any(|&x| (x - 9.5).abs() < 0.01),
        "building 4 should have x=9.5 vertex, got {:?}",
        b4_x_values
    );

    // Verify building 4 is at y=0 (row 0)
    let b4_y_values: Vec<f32> = b4_vertices.iter().map(|(_, y, _)| *y).collect();
    assert!(
        b4_y_values.iter().any(|&y| y.abs() < 0.01),
        "building 4 should have y=0 vertex, got {:?}",
        b4_y_values
    );
}

#[test]
fn render_obj_height_calculation() {
    // Test height = (lines / 10.0).max(0.5)
    // - lines=3: 3/10 = 0.3, clamped to 0.5
    // - lines=100: 100/10 = 10.0

    let rows = vec![
        make_file_row("small.rs", 1, 3),   // height should be 0.5 (clamped)
        make_file_row("large.rs", 1, 100), // height should be 10.0
    ];

    let derived = make_derived_report(rows);
    let receipt = make_receipt(derived);

    let output = render(&receipt, AnalysisFormat::Obj).expect("render OBJ");
    let obj_text = match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text output for OBJ"),
    };

    let vertices: Vec<(f32, f32, f32)> = obj_text
        .lines()
        .filter(|l| l.starts_with("v "))
        .map(|l| {
            let parts: Vec<f32> = l[2..]
                .split_whitespace()
                .map(|p| p.parse().unwrap())
                .collect();
            (parts[0], parts[1], parts[2])
        })
        .collect();

    // Building 0 (small.rs) - height should be 0.5 (clamped from 0.3)
    let b0_z_values: Vec<f32> = vertices[0..8].iter().map(|(_, _, z)| *z).collect();
    assert!(
        b0_z_values.iter().any(|&z| (z - 0.5).abs() < 0.01),
        "small.rs building should have max z=0.5, got {:?}",
        b0_z_values
    );
    assert!(
        b0_z_values.iter().all(|&z| z <= 0.51),
        "small.rs building z values should not exceed 0.5, got {:?}",
        b0_z_values
    );

    // Building 1 (large.rs) - height should be 10.0
    let b1_z_values: Vec<f32> = vertices[8..16].iter().map(|(_, _, z)| *z).collect();
    assert!(
        b1_z_values.iter().any(|&z| (z - 10.0).abs() < 0.01),
        "large.rs building should have max z=10.0, got {:?}",
        b1_z_values
    );
}

// =============================================================================
// render_midi tests
// =============================================================================

#[test]
fn render_midi_key_calculation() {
    // Test key = 60 + (depth % 12)
    // - depth=5: 60 + 5 = 65
    // - depth=15: 60 + (15 % 12) = 60 + 3 = 63 (wrap test)

    let rows = vec![
        make_file_row("depth5.rs", 5, 50),
        make_file_row("depth15.rs", 15, 50),
    ];

    let derived = make_derived_report(rows);
    let receipt = make_receipt(derived);

    let output = render(&receipt, AnalysisFormat::Midi).expect("render MIDI");
    let midi_bytes = match output {
        RenderedOutput::Binary(b) => b,
        RenderedOutput::Text(_) => panic!("expected binary output for MIDI"),
    };

    // Parse MIDI using midly
    let smf = midly::Smf::parse(&midi_bytes).expect("parse MIDI");

    // Collect all NoteOn events
    let note_ons: Vec<(u8, u8)> = smf
        .tracks
        .iter()
        .flat_map(|track| {
            track.iter().filter_map(|event| {
                if let midly::TrackEventKind::Midi {
                    message: midly::MidiMessage::NoteOn { key, vel },
                    ..
                } = event.kind
                {
                    Some((key.as_int(), vel.as_int()))
                } else {
                    None
                }
            })
        })
        .collect();

    assert_eq!(note_ons.len(), 2, "expected 2 note-on events");

    // depth=5: key = 60 + 5 = 65
    assert!(
        note_ons.iter().any(|(k, _)| *k == 65),
        "expected key=65 for depth=5, got {:?}",
        note_ons
    );

    // depth=15: key = 60 + (15 % 12) = 60 + 3 = 63
    assert!(
        note_ons.iter().any(|(k, _)| *k == 63),
        "expected key=63 for depth=15 (wrap), got {:?}",
        note_ons
    );
}

#[test]
fn render_midi_velocity_calculation() {
    // Test velocity = (40 + (lines.min(127) / 2)).min(120)
    // - lines=60: 40 + (60/2) = 40 + 30 = 70
    // - lines=200: 40 + (min(200,127)/2) = 40 + 63 = 103

    let rows = vec![
        make_file_row("lines60.rs", 1, 60),
        make_file_row("lines200.rs", 1, 200),
    ];

    let derived = make_derived_report(rows);
    let receipt = make_receipt(derived);

    let output = render(&receipt, AnalysisFormat::Midi).expect("render MIDI");
    let midi_bytes = match output {
        RenderedOutput::Binary(b) => b,
        RenderedOutput::Text(_) => panic!("expected binary output for MIDI"),
    };

    let smf = midly::Smf::parse(&midi_bytes).expect("parse MIDI");

    let velocities: Vec<u8> = smf
        .tracks
        .iter()
        .flat_map(|track| {
            track.iter().filter_map(|event| {
                if let midly::TrackEventKind::Midi {
                    message: midly::MidiMessage::NoteOn { vel, .. },
                    ..
                } = event.kind
                {
                    Some(vel.as_int())
                } else {
                    None
                }
            })
        })
        .collect();

    assert_eq!(velocities.len(), 2, "expected 2 note-on events");

    // lines=60: velocity = 40 + 30 = 70
    assert!(
        velocities.contains(&70),
        "expected velocity=70 for lines=60, got {:?}",
        velocities
    );

    // lines=200: velocity = 40 + (127/2) = 40 + 63 = 103
    assert!(
        velocities.contains(&103),
        "expected velocity=103 for lines=200, got {:?}",
        velocities
    );
}

#[test]
fn render_midi_start_time() {
    // Test start = (idx as u32) * 240
    // We'll verify that notes have increasing delta times

    let rows: Vec<FileStatRow> = (0..3)
        .map(|i| make_file_row(&format!("file{}.rs", i), 1, 50))
        .collect();

    let derived = make_derived_report(rows);
    let receipt = make_receipt(derived);

    let output = render(&receipt, AnalysisFormat::Midi).expect("render MIDI");
    let midi_bytes = match output {
        RenderedOutput::Binary(b) => b,
        RenderedOutput::Text(_) => panic!("expected binary output for MIDI"),
    };

    let smf = midly::Smf::parse(&midi_bytes).expect("parse MIDI");

    // Collect note-on absolute times
    // The start times should be 0, 240, 480 for indices 0, 1, 2
    let mut note_on_times: Vec<u32> = Vec::new();

    for track in &smf.tracks {
        let mut absolute_time = 0u32;
        for event in track {
            absolute_time += event.delta.as_int();
            if let midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            } = event.kind
            {
                note_on_times.push(absolute_time);
            }
        }
    }

    assert_eq!(note_on_times.len(), 3, "expected 3 note-on events");

    // Verify the time differences are 240 ticks apart
    // idx=0: start=0, idx=1: start=240, idx=2: start=480
    assert_eq!(note_on_times[0], 0, "first note should start at 0");
    assert_eq!(
        note_on_times[1] - note_on_times[0],
        240,
        "second note should be 240 ticks after first"
    );
    assert_eq!(
        note_on_times[2] - note_on_times[1],
        240,
        "third note should be 240 ticks after second"
    );
}
