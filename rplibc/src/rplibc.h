
void putc(char c);
char getc();

static inline void puts(char *str) {
    char *c = str;
    while (*c) {
        putc(*c);
        c++;
    }
}

static inline int atoi(char* str)
{
    int i;
    int r = 0;
    for (i = 0; str[i] != '\0'; ++i)
        r = r * 10 + str[i] - '0';
    return r;
}
