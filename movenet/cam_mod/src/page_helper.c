// A simple tester to try out kernel-space functions.

#include <linux/module.h>       /* Needed by all modules */
#include <linux/init.h>         /* Needed for the macros */
#include <asm/page.h>
#include <asm/page_types.h>

// Module metadata
MODULE_AUTHOR("Yibo Yan");
MODULE_DESCRIPTION("C kernel ext for bridging rust page helper");
MODULE_LICENSE("GPL");

unsigned long get_page_offset(void)
{
	return PAGE_OFFSET;
}
EXPORT_SYMBOL(get_page_offset);

void* get_pfn_to_virt(unsigned long long pfn) 
{
	return pfn_to_kaddr(pfn);
	// return page_to_virt(pfn_to_page(pfn));
}
EXPORT_SYMBOL(get_pfn_to_virt);

static int __init kernel_ext_init(void)
{
	printk("Loaded C Page Helper\n");
	printk(KERN_INFO "Message: %lu\n", PAGE_OFFSET);
	return 0;
}

static void __exit kernel_ext_exit(void)
{
	printk("Unloaded C Page Helper\n");
}

module_init(kernel_ext_init);
module_exit(kernel_ext_exit);
