#ifndef TS_WINDOW_H
#define TS_WINDOW_H

#include <GLFW/glfw3.h>

typedef struct {
  GLFWwindow *window;
} TS_Window;

void *ts_create_TS_Window();
void ts_destroy_TS_Window(void *window);

#endif
