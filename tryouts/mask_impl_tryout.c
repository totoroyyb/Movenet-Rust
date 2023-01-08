#include <stdio.h>
#include <stdint.h>
#include <linux/types.h>

#define PM_ENTRY_BYTES      sizeof(uint64_t)
#define PM_STATUS_BITS      3
#define PM_STATUS_OFFSET    (64 - PM_STATUS_BITS)
#define PM_STATUS_MASK      (((1LL << PM_STATUS_BITS) - 1) << PM_STATUS_OFFSET)
#define PM_STATUS(nr)       (((nr) << PM_STATUS_OFFSET) & PM_STATUS_MASK)
#define PM_PSHIFT_BITS      6
#define PM_PSHIFT_OFFSET    (PM_STATUS_OFFSET - PM_PSHIFT_BITS)
#define PM_PSHIFT_MASK      (((1LL << PM_PSHIFT_BITS) - 1) << PM_PSHIFT_OFFSET)
#define PM_PSHIFT(x)        (((uint64_t) (x) << PM_PSHIFT_OFFSET) & PM_PSHIFT_MASK)
#define PM_PFRAME_MASK      ((1LL << PM_PSHIFT_OFFSET) - 1)
#define PM_PFRAME(x)        ((x) & PM_PFRAME_MASK)

#define PM_PRESENT          PM_STATUS(4LL)
#define PM_SWAP             PM_STATUS(2LL)

int main() {
	printf("PM_ENTRY_BYTES       %ld\n", PM_ENTRY_BYTES);
	printf("PM_STATUS_BITS       %d\n", PM_STATUS_BITS);
	printf("PM_STATUS_OFFSET     %d\n", PM_STATUS_OFFSET);
	printf("PM_STATUS_MASK       %lld\n", PM_STATUS_MASK);
	printf("PM_STATUS(nr)        %lld\n", PM_STATUS(2LL));
	printf("PM_PSHIFT_BITS       %d\n", PM_PSHIFT_BITS);
	printf("PM_PSHIFT_OFFSET     %d\n", PM_PSHIFT_OFFSET);
	printf("PM_PSHIFT_MASK       %lld\n", PM_PSHIFT_MASK);
	printf("PM_PSHIFT(x)         %lld\n", PM_PSHIFT(2LL));
	printf("PM_PFRAME_MASK       %lld\n", PM_PFRAME_MASK);
	printf("PM_PFRAME(x)         %lld\n", PM_PFRAME(2LL));
	printf("PM_PRESENT           %lld\n", PM_PRESENT);
	printf("PM_SWAP              %lld\n", PM_SWAP);
	return 0;
}
