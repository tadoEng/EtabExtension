use ext_calc::output::CalcOutput;

use super::pages::build_report_pages;

pub(super) fn append(doc: &mut String, calc: &CalcOutput) {
    doc.push_str("\n// ext-report page sequence: registry\n");

    for (idx, page) in build_report_pages(calc).iter().enumerate() {
        if idx > 0 {
            doc.push_str("#pagebreak()\n");
        }
        doc.push_str("// page: ");
        doc.push_str(page.id.as_str());
        doc.push_str(" | heading: ");
        doc.push_str(page.heading);
        doc.push_str(" | layout: ");
        doc.push_str(page.layout.label());
        doc.push_str(" | availability: ");
        doc.push_str(page.availability.label());
        if !page.data_files.is_empty() {
            doc.push_str(" | data: ");
            doc.push_str(&page.data_files.join(", "));
        }
        if !page.image_files.is_empty() {
            doc.push_str(" | images: ");
            doc.push_str(&page.image_files.join(", "));
        }
        doc.push('\n');
        doc.push_str(page.typst_call.source());
        doc.push_str("\n\n");
    }
}
