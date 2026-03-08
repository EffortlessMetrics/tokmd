use criterion::{Criterion, criterion_group, criterion_main};
use tokmd_types::cockpit::*;

fn create_receipt() -> CockpitReceipt {
    CockpitReceipt {
        schema_version: 3,
        mode: "PR".to_string(),
        generated_at_ms: 1234567890,
        base_ref: "main".to_string(),
        head_ref: "feat".to_string(),
        change_surface: ChangeSurface {
            files_changed: 10,
            insertions: 200,
            deletions: 50,
            net_lines: 150,
            commits: 1,
            churn_velocity: 0.0,
            change_concentration: 0.0,
        },
        composition: Composition {
            code_pct: 0.8,
            test_pct: 0.1,
            docs_pct: 0.05,
            config_pct: 0.05,
            test_ratio: 0.125,
        },
        code_health: CodeHealth {
            score: 85,
            grade: "A".to_string(),
            large_files_touched: 0,
            avg_file_size: 0,
            complexity_indicator: ComplexityIndicator::Low,
            warnings: vec![],
        },
        risk: Risk {
            score: 20,
            level: RiskLevel::Low,
            hotspots_touched: vec![],
            bus_factor_warnings: vec![],
        },
        contracts: Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        },
        evidence: Evidence {
            overall_status: GateStatus::Pass,
            mutation: MutationGate {
                meta: GateMeta {
                    status: GateStatus::Pass,
                    source: EvidenceSource::RanLocal,
                    commit_match: CommitMatch::Exact,
                    scope: ScopeCoverage {
                        relevant: vec![],
                        tested: vec![],
                        ratio: 1.0,
                        lines_relevant: None,
                        lines_tested: None,
                    },
                    evidence_commit: None,
                    evidence_generated_at_ms: None,
                },
                survivors: vec![],
                killed: 100,
                timeout: 0,
                unviable: 0,
            },
            diff_coverage: None,
            contracts: None,
            supply_chain: None,
            determinism: None,
            complexity: None,
        },
        review_plan: vec![],
        trend: None,
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let receipt = create_receipt();
    c.bench_function("render_markdown", |b| {
        b.iter(|| tokmd_cockpit::render::render_markdown(std::hint::black_box(&receipt)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
