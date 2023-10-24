typedef unsigned long size_t;

void putc(char c);
char getc();
int open(const char* path, int flags);
size_t read(int fd, void* buf, size_t cnt);
size_t write(int fd, void* buf, size_t cnt); 
int close(int fd);

#define O_RDONLY     0x00010000
#define O_WRONLY     0x00020000
#define O_RDWR       0x00030000
#define O_NONBLOCK   0x00040000
#define O_APPEND     0x00080000
#define O_SHLOCK     0x00100000
#define O_EXLOCK     0x00200000
#define O_ASYNC      0x00400000
#define O_FSYNC      0x00800000
#define O_CLOEXEC    0x01000000
#define O_CREAT      0x02000000
#define O_TRUNC      0x04000000
#define O_EXCL       0x08000000
#define O_DIRECTORY  0x10000000
#define O_STAT       0x20000000
#define O_SYMLINK    0x40000000
#define O_NOFOLLOW   0x80000000
#define O_ACCMODE    (O_RDONLY | O_WRONLY | O_RDWR)

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
