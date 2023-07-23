
void putc(char c);
char getc();

static inline void puts(char *str) {
    char *c = str;
    while (*c) {
        putc(*c);
        c++;
    }
}
