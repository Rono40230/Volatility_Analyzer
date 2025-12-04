use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

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
        PdfDocument::new("Formules Straddle", Mm(210.0), Mm(297.0), "Layer 1");

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
        "Formules & Calculs - Straddle Trading",
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
