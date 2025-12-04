// commands/global_analysis_commands.rs - Commandes pour l'analyse globale
use crate::models::{AnalysisFilters, GlobalAnalysisResult};
use crate::services::{ArchiveService, GlobalAnalyzer};
use tauri::State;

#[tauri::command]
pub async fn analyze_all_archives(
    filters: Option<AnalysisFilters>,
    archive_service: State<'_, ArchiveService>,
) -> Result<GlobalAnalysisResult, String> {
    // On clone le service pour le passer au GlobalAnalyzer
    // ArchiveService est léger (contient juste un pool DB)
    let global_analyzer = GlobalAnalyzer::new((*archive_service).clone());

    // L'analyse peut être longue, donc on la lance en async (même si ici c'est bloquant pour le thread)
    // Pour de très gros volumes, on utiliserait spawn_blocking
    global_analyzer.analyze_all_archives(filters)
}
