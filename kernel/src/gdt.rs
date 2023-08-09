use spin::Lazy;
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::instructions::tables::load_tss;
use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

struct Selectors
{
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector
}

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| 
{
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] =
    {
        const STACK_SIZE: usize = 4096 * 5;
        static STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        VirtAddr::from_ptr(&STACK) + STACK_SIZE
    };
    tss
});

static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(||
{
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
    (gdt, Selectors{code_selector, data_selector, tss_selector})
});

pub fn init()
{
    GDT.0.load();

    unsafe
    {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        ES::set_reg(GDT.1.data_selector);
        SS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}