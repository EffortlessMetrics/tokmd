use tokmd_analysis_types::{
    ApiSurfaceReport, Archetype, AssetReport, ComplexityReport, CorporateFingerprint,
    DependencyReport, DuplicateReport, EffortEstimateReport, EntropyReport, FunReport, GitReport,
    ImportReport, LicenseReport, PredictiveChurnReport, TopicClouds,
};

#[derive(Debug, Default)]
pub(super) struct AnalysisReports {
    pub(super) archetype: Option<Archetype>,
    pub(super) topics: Option<TopicClouds>,
    pub(super) entropy: Option<EntropyReport>,
    pub(super) predictive_churn: Option<PredictiveChurnReport>,
    pub(super) corporate_fingerprint: Option<CorporateFingerprint>,
    pub(super) license: Option<LicenseReport>,
    pub(super) assets: Option<AssetReport>,
    pub(super) deps: Option<DependencyReport>,
    pub(super) git: Option<GitReport>,
    pub(super) imports: Option<ImportReport>,
    pub(super) dup: Option<DuplicateReport>,
    pub(super) complexity: Option<ComplexityReport>,
    pub(super) api_surface: Option<ApiSurfaceReport>,
    pub(super) effort: Option<EffortEstimateReport>,
    pub(super) fun: Option<FunReport>,
}
