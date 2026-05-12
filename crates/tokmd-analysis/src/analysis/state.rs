use tokmd_analysis_types::{
    ApiSurfaceReport, Archetype, AssetReport, ComplexityReport, CorporateFingerprint,
    DependencyReport, DuplicateReport, EffortEstimateReport, EntropyReport, FunReport, GitReport,
    ImportReport, LicenseReport, PredictiveChurnReport, TopicClouds,
};

#[derive(Debug, Default)]
pub(super) struct AnalysisOutputs {
    pub assets: Option<AssetReport>,
    pub deps: Option<DependencyReport>,
    pub imports: Option<ImportReport>,
    pub dup: Option<DuplicateReport>,
    pub git: Option<GitReport>,
    pub churn: Option<PredictiveChurnReport>,
    pub fingerprint: Option<CorporateFingerprint>,
    pub entropy: Option<EntropyReport>,
    pub license: Option<LicenseReport>,
    pub complexity: Option<ComplexityReport>,
    pub api_surface: Option<ApiSurfaceReport>,
    pub archetype: Option<Archetype>,
    pub topics: Option<TopicClouds>,
    pub effort: Option<EffortEstimateReport>,
    pub fun: Option<FunReport>,
}
