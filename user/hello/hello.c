#include <stdint.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>
#include <stdio.h>
#include <sys/wait.h>

#include "syslib.h"

void delay_cycles() {
    for (int i = 0; i < 1000000000; i++) {
        asm volatile ("nop");
    }
}

int main() {
    fork();
    for (int i = 0; i < 5; i++) {
        printf("%d: mypid: %d\n", i, getpid());
        delay_cycles();
    }
}
