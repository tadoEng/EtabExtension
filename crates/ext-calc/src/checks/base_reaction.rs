use anyhow::Result;
use crate::{code_params::CodeParams, output::{BaseReactionsOutput, BaseReactionDir, Quantity}};
use crate::tables::base_reactions::BaseReactionRow;

pub fn run(
    _reactions: &[BaseReactionRow],
    params: &CodeParams,
) -> Result<BaseReactionsOutput> {
    Ok(BaseReactionsOutput {
        rows: vec![],
        direction_x: BaseReactionDir {
            rsa_case: String::new(),
            elf_case: String::new(),
            v_rsa: Quantity::new(0.0, params.unit_context.force_label()),
            v_elf: Quantity::new(0.0, params.unit_context.force_label()),
            ratio: 0.0,
            pass: true,
        },
        direction_y: BaseReactionDir {
            rsa_case: String::new(),
            elf_case: String::new(),
            v_rsa: Quantity::new(0.0, params.unit_context.force_label()),
            v_elf: Quantity::new(0.0, params.unit_context.force_label()),
            ratio: 0.0,
            pass: true,
        },
    })
}
