use tokmd_analysis_types::{
    ApiSurfaceReport, Archetype, AssetReport, ComplexityReport, CorporateFingerprint,
    DependencyReport, DuplicateReport, EntropyReport, FunReport, GitReport, ImportReport,
    LicenseReport, PredictiveChurnReport, TopicClouds,
};

#[derive(Debug, Default)]
pub(super) struct AnalysisOutputs {
    pub(super) assets: Option<AssetReport>,
    pub(super) deps: Option<DependencyReport>,
    pub(super) imports: Option<ImportReport>,
    pub(super) dup: Option<DuplicateReport>,
    pub(super) git: Option<GitReport>,
    pub(super) churn: Option<PredictiveChurnReport>,
    pub(super) fingerprint: Option<CorporateFingerprint>,
    pub(super) entropy: Option<EntropyReport>,
    pub(super) license: Option<LicenseReport>,
    pub(super) complexity: Option<ComplexityReport>,
    pub(super) api_surface: Option<ApiSurfaceReport>,
    pub(super) archetype: Option<Archetype>,
    pub(super) topics: Option<TopicClouds>,
    pub(super) fun: Option<FunReport>,
}
