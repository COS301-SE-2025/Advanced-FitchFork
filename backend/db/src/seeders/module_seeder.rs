use crate::factories::module_factory;
use sqlx::SqlitePool;

pub async fn seed(pool: &SqlitePool) {
    log::info!("Seeding modules...");

    // Original modules
    module_factory::make("COS314", 2025, Some("Artificial Intelligence"), 16, pool).await;
    module_factory::make("COS333", 2025, Some("Software Engineering"), 16, pool).await;
    module_factory::make("COS301", 2025, Some("Something"), 16, pool).await;

    // Varied and expanded modules
    module_factory::make("INF272", 2024, Some("Information Systems Infrastructure"), 12, pool).await;
    module_factory::make("MIT300", 2025, Some("Management of IT Projects"), 20, pool).await;
    module_factory::make("CSC140", 2023, Some("Computational Thinking"), 8, pool).await;
    module_factory::make("BIO101", 2022, Some("Introduction to Biology"), 10, pool).await;
    module_factory::make("PHY201", 2025, Some("Electromagnetism and Optics"), 14, pool).await;
    module_factory::make("MTH310", 2023, Some("Advanced Calculus"), 18, pool).await;
    module_factory::make("DSC200", 2024, Some("Data Science Foundations"), 16, pool).await;
    module_factory::make("ENG120", 2022, Some("Academic English Writing"), 6, pool).await;
    module_factory::make("ENV150", 2025, Some("Environmental Science and Policy"), 12, pool).await;
    module_factory::make("LAW310", 2024, Some("Cyberlaw and Ethics"), 10, pool).await;
    module_factory::make("PSY202", 2023, Some("Cognitive Psychology"), 12, pool).await;
    module_factory::make("HCI400", 2025, Some("Human-Computer Interaction"), 16, pool).await;
    module_factory::make("FIN210", 2023, Some("Financial Accounting Basics"), 12, pool).await;
    module_factory::make("BUS101", 2024, Some("Introduction to Business Management"), 10, pool).await;
    module_factory::make("ARC320", 2025, Some("Architectural Design Studio III"), 24, pool).await;
    module_factory::make("MUS100", 2022, Some("Fundamentals of Music Theory"), 8, pool).await;
    module_factory::make("HIS205", 2024, Some("History of Modern Africa"), 10, pool).await;
    module_factory::make("PHL150", 2023, Some("Logic and Critical Reasoning"), 8, pool).await;
    module_factory::make("CSC420", 2025, Some("Advanced Topics in Machine Learning"), 20, pool).await;
    module_factory::make("ART103", 2022, Some("Visual Communication and Design"), 10, pool).await;
    module_factory::make("MED200", 2024, Some("Medical Terminology"), 6, pool).await;
    module_factory::make("CHE210", 2025, Some("Organic Chemistry I"), 16, pool).await;
    module_factory::make("SOC110", 2023, Some("Introduction to Sociology"), 10, pool).await;

    log::info!("Modules seeded.");
}
