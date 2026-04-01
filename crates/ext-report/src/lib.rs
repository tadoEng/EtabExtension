use anyhow::Result;
use ext_calc::output::CalcOutput;

#[derive(Debug, Clone)]
pub struct ReportProjectMeta {
    pub project_name: String,
    pub reference: String,
    pub engineer: String,
    pub date: String,
    pub subject: String,
}

#[derive(Debug, Clone)]
pub struct ReportInput {
    pub project: ReportProjectMeta,
    pub calc: CalcOutput,
    pub images: Vec<String>,
}

pub fn build_typst_document(input: &ReportInput) -> String {
    let mut doc = String::new();
    doc.push_str("#set text(font: \"Arial\", size: 10pt)\n");
    doc.push_str("#set page(width: 17in, height: 11in, margin: 0.5in)\n\n");
    doc.push_str(&format!(
        "= {}\n\n",
        input.project.project_name.replace('\n', " ")
    ));
    doc.push_str(&format!(
        "*Reference:* {}  \\\n*Engineer:* {}  \\\n*Subject:* {}\n\n",
        input.project.reference, input.project.engineer, input.project.subject
    ));
    doc.push_str("== Summary\n\n");
    doc.push_str(&format!(
        "- Overall status: {}\n- Checks: {}\n- Passed: {}\n- Failed: {}\n",
        input.calc.summary.overall_status,
        input.calc.summary.check_count,
        input.calc.summary.pass_count,
        input.calc.summary.fail_count
    ));
    if !input.calc.summary.lines.is_empty() {
        doc.push_str("\n== Loaded Inputs\n\n");
        for line in &input.calc.summary.lines {
            doc.push_str(&format!("- {}: {}\n", line.key, line.message));
        }
    }
    doc
}

pub fn compile_pdf(_input: &ReportInput) -> Result<Vec<u8>> {
    anyhow::bail!("Typst PDF compilation is not wired yet; build_typst_document is ready for integration")
}
