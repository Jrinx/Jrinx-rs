use fdt::node::FdtNode;
use jrinx_devprober::devprober;
use jrinx_error::Result;

#[devprober(compatible = "ns16550a")]
fn probe(_node: &FdtNode) -> Result<()> {
    // TODO
    Ok(())
}
