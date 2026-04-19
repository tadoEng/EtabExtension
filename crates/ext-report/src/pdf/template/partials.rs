use std::path::Path;

use anyhow::{Context, Result};

pub(crate) struct TypstPartial {
    pub path: &'static str,
    pub source: &'static str,
}

pub(crate) const TYPST_PARTIALS: &[TypstPartial] = &[
    TypstPartial {
        path: "page_setup.typ",
        source: include_str!("partials/page_setup.typ"),
    },
    TypstPartial {
        path: "styles.typ",
        source: include_str!("partials/styles.typ"),
    },
    TypstPartial {
        path: "components.typ",
        source: include_str!("partials/components.typ"),
    },
    TypstPartial {
        path: "layouts.typ",
        source: include_str!("partials/layouts.typ"),
    },
];

pub(super) fn append_all(doc: &mut String) {
    for partial in TYPST_PARTIALS {
        doc.push_str("\n// ext-report partial: ");
        doc.push_str(partial.path);
        doc.push('\n');
        doc.push_str(partial.source);
        doc.push('\n');
    }
}

pub(crate) fn write_all_to_dir(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)
        .with_context(|| format!("Failed to create Typst debug directory {}", dir.display()))?;
    for partial in TYPST_PARTIALS {
        let path = dir.join(partial.path);
        std::fs::write(&path, partial.source)
            .with_context(|| format!("Failed to write Typst partial {}", path.display()))?;
    }
    Ok(())
}
