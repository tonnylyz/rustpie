#include <rplibc.h>

int main(int argc, char** argv) {
    int i;
    puts("hello world from C arguments:\n");
    for (i = 0; i < argc; i++) {
        puts(argv[i]);
        putc('\n');
    }
    return 0;
}
