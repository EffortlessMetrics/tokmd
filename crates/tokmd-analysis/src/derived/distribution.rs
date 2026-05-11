use tokmd_analysis_types::{DistributionReport, HistogramBucket};
use tokmd_scan::{gini_coefficient, percentile, round_f64};
use tokmd_types::FileRow;

pub(super) fn build_distribution_report(rows: &[&FileRow]) -> DistributionReport {
    let mut sizes: Vec<usize> = rows.iter().map(|r| r.lines).collect();
    sizes.sort();

    if sizes.is_empty() {
        return DistributionReport {
            count: 0,
            min: 0,
            max: 0,
            mean: 0.0,
            median: 0.0,
            p90: 0.0,
            p99: 0.0,
            gini: 0.0,
        };
    }

    let count = sizes.len();
    let sum: usize = sizes.iter().sum();
    let mean = sum as f64 / count as f64;
    let median = if count % 2 == 1 {
        sizes[count / 2] as f64
    } else {
        (sizes[count / 2 - 1] as f64 + sizes[count / 2] as f64) / 2.0
    };
    let p90 = percentile(&sizes, 0.90);
    let p99 = percentile(&sizes, 0.99);
    let gini = gini_coefficient(&sizes);

    DistributionReport {
        count,
        min: *sizes.first().unwrap_or(&0),
        max: *sizes.last().unwrap_or(&0),
        mean: round_f64(mean, 2),
        median: round_f64(median, 2),
        p90: round_f64(p90, 2),
        p99: round_f64(p99, 2),
        gini: round_f64(gini, 4),
    }
}

pub(super) fn build_histogram(rows: &[&FileRow]) -> Vec<HistogramBucket> {
    let total = rows.len();
    let buckets = vec![
        ("Tiny", 0, Some(50)),
        ("Small", 51, Some(200)),
        ("Medium", 201, Some(500)),
        ("Large", 501, Some(1000)),
        ("Huge", 1001, None),
    ];

    let mut counts = vec![0usize; buckets.len()];
    for row in rows {
        let size = row.lines;
        for (idx, (_label, min, max)) in buckets.iter().enumerate() {
            let in_range = if let Some(max) = max {
                size >= *min && size <= *max
            } else {
                size >= *min
            };
            if in_range {
                counts[idx] += 1;
                break;
            }
        }
    }

    buckets
        .into_iter()
        .zip(counts)
        .map(|((label, min, max), files)| HistogramBucket {
            label: label.to_string(),
            min,
            max,
            files,
            pct: if total == 0 {
                0.0
            } else {
                round_f64(files as f64 / total as f64, 4)
            },
        })
        .collect()
}
