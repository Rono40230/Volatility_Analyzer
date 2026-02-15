use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::collections::BTreeMap;
use tauri::{State, AppHandle, Manager};
use crate::services::ArchiveService;
use crate::models::Archive;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FormuleData {
    pub titre: String,
    pub definition: String,
    pub formule: String,
    pub inputs: Vec<String>,
    pub output: OutputData,
    pub exemple: String,
    pub notes: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct OutputData {
    pub r#type: String,
    pub range: String,
    pub unite: String,
}

#[tauri::command]
pub fn exporter_formules_pdf(
    formules: Vec<FormuleData>,
    fichier_sortie: String,
) -> Result<String, String> {
    let path = Path::new(&fichier_sortie);

    // Créer le fichier
    let file = File::create(path).map_err(|e| format!("Erreur création fichier: {}", e))?;

    let (document, page1, layer1) =
        PdfDocument::new("Formules Volatilité", Mm(210.0), Mm(297.0), "Layer 1");

    let font = document
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("Erreur font: {:?}", e))?;

    let font_bold = document
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("Erreur font bold: {:?}", e))?;

    let mut current_page = page1;
    let mut current_layer_id = layer1;
    let mut y_pos = 280.0;
    let margin = 10.0;

    // Titre principal
    let current_layer = document.get_page(current_page).get_layer(current_layer_id);
    current_layer.use_text(
        "Formules & Calculs - Analyse Volatilité",
        18.0,
        Mm(margin),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 15.0;

    for formule in formules {
        // Vérifier si besoin d'une nouvelle page
        if y_pos < 40.0 {
            let (new_page, new_layer) = document.add_page(Mm(210.0), Mm(297.0), "Page");
            current_page = new_page;
            current_layer_id = new_layer;
            y_pos = 280.0;
        }

        let current_layer = document.get_page(current_page).get_layer(current_layer_id);

        // Titre formule
        current_layer.use_text(&formule.titre, 14.0, Mm(margin), Mm(y_pos), &font_bold);
        y_pos -= 6.0;

        // Définition
        current_layer.use_text(&formule.definition, 10.0, Mm(margin), Mm(y_pos), &font);
        y_pos -= 6.0;

        // Formule
        current_layer.use_text("Formule:", 11.0, Mm(margin), Mm(y_pos), &font_bold);
        y_pos -= 5.0;
        current_layer.use_text(&formule.formule, 9.0, Mm(margin + 3.0), Mm(y_pos), &font);
        y_pos -= 6.0;

        // Inputs
        current_layer.use_text("Inputs:", 11.0, Mm(margin), Mm(y_pos), &font_bold);
        y_pos -= 5.0;
        for input in &formule.inputs {
            current_layer.use_text(
                format!("• {}", input),
                9.0,
                Mm(margin + 3.0),
                Mm(y_pos),
                &font,
            );
            y_pos -= 4.0;
        }
        y_pos -= 3.0;

        // Output
        current_layer.use_text("Output:", 11.0, Mm(margin), Mm(y_pos), &font_bold);
        y_pos -= 5.0;
        current_layer.use_text(
            format!("Type: {}", formule.output.r#type),
            9.0,
            Mm(margin + 3.0),
            Mm(y_pos),
            &font,
        );
        y_pos -= 4.0;
        current_layer.use_text(
            format!("Range: {}", formule.output.range),
            9.0,
            Mm(margin + 3.0),
            Mm(y_pos),
            &font,
        );
        y_pos -= 4.0;
        current_layer.use_text(
            format!("Unité: {}", formule.output.unite),
            9.0,
            Mm(margin + 3.0),
            Mm(y_pos),
            &font,
        );
        y_pos -= 6.0;

        // Exemple
        current_layer.use_text("Exemple:", 11.0, Mm(margin), Mm(y_pos), &font_bold);
        y_pos -= 5.0;
        current_layer.use_text(&formule.exemple, 9.0, Mm(margin + 3.0), Mm(y_pos), &font);
        y_pos -= 6.0;

        // Notes
        if !formule.notes.is_empty() {
            current_layer.use_text("Notes:", 11.0, Mm(margin), Mm(y_pos), &font_bold);
            y_pos -= 5.0;
            for note in &formule.notes {
                current_layer.use_text(
                    format!("• {}", note),
                    9.0,
                    Mm(margin + 3.0),
                    Mm(y_pos),
                    &font,
                );
                y_pos -= 4.0;
            }
        }

        y_pos -= 6.0;
    }

    // Sauvegarder le PDF
    document
        .save(&mut BufWriter::new(file))
        .map_err(|e| format!("Erreur save PDF: {:?}", e))?;

    Ok(format!("PDF exporté: {}", fichier_sortie))
}

#[tauri::command]
pub fn export_comparative_analysis_pdf(
    app: AppHandle,
    archive_service: State<'_, ArchiveService>,
    output_path: String,
) -> Result<String, String> {
    // 1. Récupérer toutes les archives
    let archives = archive_service.list_archives()?;
    if archives.is_empty() { return Err("Aucune archive disponible".to_string()); }

    // 2. Filtrage des archives par type
    let mut entry_archives: Vec<&Archive> = Vec::new();
    let mut correlation_archives: Vec<&Archive> = Vec::new();

    let mut min_date = String::from("9999-99-99");
    let mut max_date = String::from("0000-00-00");

    for archive in &archives {
        // Mise à jour de la période globale
        if archive.period_start < min_date { min_date = archive.period_start.clone(); }
        if archive.period_end > max_date { max_date = archive.period_end.clone(); }

        // Classification simplifiée
        if archive.archive_type.contains("Point d'Entrée") {
            entry_archives.push(archive);
        } else if archive.archive_type.contains("Correlation") {
            correlation_archives.push(archive);
        }
    }

    // 3. Setup PDF
    let final_path = if output_path == "TEMP_PREVIEW" {
        // En mode preview, on sauvegarde dans le dossier de téléchargement avec un nom temporaire
        let download_dir = app.path().download_dir().map_err(|e| format!("Erreur chemin download: {}", e))?;
        let mut path = download_dir;
        path.push("preview_volatility_analysis.pdf");
        path.to_string_lossy().to_string()
    } else {
        output_path.clone()
    };
    
    // A4 Paysage : Largeur 297mm, Hauteur 210mm
    let (document, page1, layer1) = PdfDocument::new("Rapport Simplifié", Mm(297.0), Mm(210.0), "Layer 1");
    
    let font = document.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let font_bold = document.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();

    let mut current_page = page1;
    let mut current_layer_id = layer1;
    let mut y = 190.0; // Start Y adapté pour la hauteur 210mm
    let margin = 15.0;

    // --- PAGE 1: SECTION 1 (Volatilité / Points d'entrée) ---
    let current_layer = document.get_page(current_page).get_layer(current_layer_id);
    
    // Titre global (compact)
    let global_title = if min_date != "9999-99-99" {
         format!("RAPPORT VOLATILITÉ & CORRELATION  ({} - {})", min_date.split(' ').next().unwrap_or(""), max_date.split(' ').next().unwrap_or(""))
    } else {
         "RAPPORT ANALYSE HISTORIQUE".to_string()
    };
    current_layer.use_text(global_title, 14.0, Mm(margin), Mm(y), &font_bold);
    y -= 15.0;

    // --- SECTION 1 ---
    current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None))); 
    current_layer.use_text("1. VOLATILITÉ BRUTE & POINTS D'ENTRÉE", 11.0, Mm(margin), Mm(y), &font_bold);
    y -= 8.0;

    // En-têtes Tableau 1
    current_layer.use_text("Paire", 9.0, Mm(margin), Mm(y), &font_bold);
    current_layer.use_text("Meilleur Point d'Entrée", 9.0, Mm(margin + 40.0), Mm(y), &font_bold);
    y -= 5.0;

    // Lignes Tableau 1
    for archive in entry_archives {
        if y < 20.0 {
            let (np, nl) = document.add_page(Mm(297.0), Mm(210.0), "Page");
            current_page = np; current_layer_id = nl; y = 190.0;
        }
        let current_layer = document.get_page(current_page).get_layer(current_layer_id);

        let pair = extract_symbol_from_title(&archive.title).unwrap_or("N/A");
        // Extraction heure depuis titre "Analyse ETHUSD 14:30" => prends le dernier mot
        let entry_point = archive.title.split_whitespace().last().unwrap_or("-");

        current_layer.use_text(pair, 8.0, Mm(margin), Mm(y), &font);
        current_layer.use_text(entry_point, 8.0, Mm(margin + 40.0), Mm(y), &font);
        y -= 4.0;
    }

    y -= 10.0; // Espace entre sections

    // --- SECTION 2 ---
    if y < 40.0 {
        let (np, nl) = document.add_page(Mm(297.0), Mm(210.0), "Page");
        current_page = np; current_layer_id = nl; y = 190.0;
    }
    let current_layer = document.get_page(current_page).get_layer(current_layer_id);

    current_layer.use_text("2. CORRELATION PAIRE / ÉVÉNEMENT", 11.0, Mm(margin), Mm(y), &font_bold);
    y -= 8.0;

    // En-têtes Tableau 2 (4 colonnes - Espacement large pour Paysage)
    // Largeur dispo ~280mm. 
    // Col 1 (Paire): 0
    // Col 2 (Event 1): 30
    // Col 3 (Event 2): 110
    // Col 4 (Event 3): 190
    current_layer.use_text("Paire", 9.0, Mm(margin), Mm(y), &font_bold);
    current_layer.use_text("Meilleur Événement", 9.0, Mm(margin + 30.0), Mm(y), &font_bold);
    current_layer.use_text("2ème Événement", 9.0, Mm(margin + 110.0), Mm(y), &font_bold);
    current_layer.use_text("3ème Événement", 9.0, Mm(margin + 190.0), Mm(y), &font_bold);
    y -= 5.0;

    // Groupement des événements par paire
    let mut events_by_pair: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for archive in correlation_archives {
        let pair = extract_symbol_from_title(&archive.title).unwrap_or("N/A").to_string();
        
        let mut event_name = truncate_string(&archive.title, 55); // Increased truncation limit
        if let Some(start) = archive.title.find("nement ") { 
             if let Some(end) = archive.title.find(" sur la") {
                 if start + 7 < end {
                    event_name = truncate_string(&archive.title[start+7..end], 55);
                 }
             }
        }
        events_by_pair.entry(pair).or_default().push(event_name);
    }

    // Lignes Tableau 2
    for (pair, events) in events_by_pair {
        if y < 20.0 {
            let (np, nl) = document.add_page(Mm(297.0), Mm(210.0), "Page");
            current_page = np; current_layer_id = nl; y = 190.0;
        }
        let current_layer = document.get_page(current_page).get_layer(current_layer_id);

        current_layer.use_text(&pair, 8.0, Mm(margin), Mm(y), &font);
        
        // Affichage des 3 premiers événements
        let text1 = events.first().map(|s| s.as_str()).unwrap_or("-");
        let text2 = events.get(1).map(|s| s.as_str()).unwrap_or("-");
        let text3 = events.get(2).map(|s| s.as_str()).unwrap_or("-");

        current_layer.use_text(text1, 8.0, Mm(margin + 30.0), Mm(y), &font);
        current_layer.use_text(text2, 8.0, Mm(margin + 110.0), Mm(y), &font);
        current_layer.use_text(text3, 8.0, Mm(margin + 190.0), Mm(y), &font);
        
        y -= 4.0;
    }

    // 4. Output Handling
    let path = Path::new(&final_path);
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    
    let file = File::create(path).map_err(|e| format!("Erreur création fichier: {}", e))?;
    document.save(&mut BufWriter::new(file)).map_err(|e| format!("Erreur PDF (file): {:?}", e))?;
    
    Ok(final_path)
}
// Fonctions helper

fn extract_symbol_from_title(title: &str) -> Option<&str> {
    // Chercher les symboles forex courants (4-7 lettres en majuscules)
    let parts: Vec<&str> = title.split_whitespace().collect();
    for part in parts {
        if part.len() >= 4 && part.len() <= 7 && part.chars().all(|c| c.is_ascii_uppercase()) {
            // Vérifier que c'est probablement une paire Forex
            if part.ends_with("USD") || part.ends_with("JPY") || part.ends_with("EUR") || 
               part.ends_with("GBP") || part.ends_with("CHF") || part.ends_with("AUD") || 
               part.ends_with("CAD") || part.ends_with("NZD") {
                return Some(part);
            }
        }
    }
    None
}

fn truncate_string(s: &str, max_len: usize) -> String {
    // Nettoyer les caractères non supportés par la police standard (emojis, etc)
    // Helvetica ne supporte que Latin-1
    let cleaned: String = s.chars()
        .filter(|c| c.is_ascii() || "éèàùçâêîôûëïüÿñÉÈÀÙÇÂÊÎÔÛËÏÜŸÑ".contains(*c))
        .collect();

    // Utiliser les itérateurs de caractères pour éviter de couper au milieu d'un caractère UTF-8
    let char_count = cleaned.chars().count();
    
    if char_count > max_len {
        let truncated: String = cleaned.chars().take(max_len).collect();
        format!("{}...", truncated)
    } else {
        cleaned
    }
}
