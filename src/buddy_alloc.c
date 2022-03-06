#include <assert.h>
#include <math.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define frag_size ((int)sizeof(struct fragment))
#define MIN_BLOCK_SIZE (32 - (frag_size))
#define LEVELS 32
#define MAGIC_VAL 22131232

#define MEMORY_SIZE (128 + 10) * 1024 * 1024

static struct fragment *mem_arr[LEVELS];
static int heap_size;
static struct fragment *mem;
static int taken_blocks;

struct fragment {
    /* Next free segment in the same level */
    struct fragment *next;
    int size;
    unsigned taken;
};

static bool get_taken(struct fragment *f) { return f->taken & 1; }

static void set_taken(struct fragment *f, bool val) {
    f->taken &= ~1;
    f->taken |= val;
}

static void swap(struct fragment **x, struct fragment **y) {
    struct fragment *t = *x;
    *x = *y;
    *y = t;
}

static bool is_block(struct fragment *f) {
    return (f->taken & 0xfffffffe) == (MAGIC_VAL & 0xfffffffe);
}

static int pow2(int num) {
    int pow = 0;
    while (num /= 2)
        pow++;
    return pow;
}

static void add_free(struct fragment *f, int i) {
    f->next = mem_arr[i];
    mem_arr[i] = f;
}

static void remove_free(struct fragment *f, int i) {
    struct fragment *walk = mem_arr[i];

    if (walk == f) {
        mem_arr[i] = f->next;
        return;
    }
    while (walk->next != f)
        walk = walk->next;
    walk->next = f->next;
}

static struct fragment *buddy_addr(struct fragment *f, int i) {
    /* Because fragment sizes are multiples of two, we can use bitwise
       operations to find buddy address | ... |   64b   |   64b    | 
                                        ^     ^         ^
                                        0     x       x + 64
        We could add 64, but we would have to be sure that our fragment is
       before buddy. If we 6th bit, if it's zero it will add 64, if it's one it
       will substract 64, so we will get the address either way.
    */
    return (struct fragment *)((((uint8_t *)f - (uint8_t *)mem) ^ (1 << i)) +
                               (uint8_t *)mem);
}

static void split(struct fragment *f, int i) {
    int new_size = (f->size + frag_size) / 2;

    f->size = new_size - frag_size;
    struct fragment *buddy = (struct fragment *)((uint8_t *)f + new_size);
    *buddy = (struct fragment){NULL, new_size - frag_size, MAGIC_VAL};
    if(!((uint8_t *)buddy - (uint8_t *)f == f->size + frag_size)) {
        assert(false);
    }
    set_taken(buddy, false);
    add_free(buddy, i - 1);
}

void heap_init() {
    void *memPool = malloc(MEMORY_SIZE);
    for (size_t i = 0; i < LEVELS; ++i)
        mem_arr[i] = NULL;
    taken_blocks = 0;
    heap_size = 0;
    mem = (struct fragment *)memPool;
    /* Try to allocate as much memory as possible */
    while (true) {
        int i = log2(MEMORY_SIZE - heap_size);
        int new_size = 1 << i;
        if (new_size < MIN_BLOCK_SIZE || heap_size + new_size > MEMORY_SIZE)
            break;
        struct fragment *rem = (struct fragment *)((uint8_t *)mem + heap_size);
        *rem = (struct fragment){NULL, new_size - frag_size, MAGIC_VAL};
        set_taken(rem, 0);
        add_free(rem, i);
        heap_size += new_size;
    }
}

void *heap_alloc(int size) {
    if (size < MIN_BLOCK_SIZE)
        size = MIN_BLOCK_SIZE;
    int i = pow2(size + frag_size) + 1;
    while (!mem_arr[i]) {
        i++;
        if (i >= LEVELS)
            return NULL;
    }
    struct fragment *walk = mem_arr[i];
    set_taken(walk, true);
    remove_free(walk, i);
    /* Try to split the block while it's size is larger than the wanted size */
    while ((walk->size - frag_size) / 2 - frag_size > size &&
           (walk->size - frag_size) / 2 > MIN_BLOCK_SIZE)
        split(walk, i--);

    taken_blocks++;
    assert(walk->size >= size);
    return walk + 1;
}

static struct fragment *merge(struct fragment *f, int i) {
    struct fragment *b = buddy_addr(f, i);
    if (b->size != f->size || get_taken(b))
        return NULL;
    if (b < f)
        swap(&b, &f);
    assert(is_block(b) && is_block(f));
    assert(f != b);

    remove_free(b, i);
    remove_free(f, i);

    f->size = 2 * f->size + frag_size;

    add_free(f, i + 1);
    return f;
}

bool heap_free(void *blk) {
    struct fragment *f = (struct fragment *)blk - 1;
    if (!blk || !is_block(f))
        return false;
    set_taken(f, false);

    int i = log2(f->size + frag_size);

    add_free(f, i);
    /* Try to merge fragments until fragment size does not exceed max heap size
     */
    while (2 * f->size + frag_size <= heap_size)
        if (!(f = merge(f, i++)))
            break;

    taken_blocks--;
    return true;
}

int heap_done() { free(mem); return taken_blocks; }

