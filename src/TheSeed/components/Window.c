#include <glad/gl.h>

#include "GL/gl.h"
#include "TheSeed/components/Window.h"
#include <GLFW/glfw3.h>
#include <stdio.h>
#include <stdlib.h>

#define WIDTH 800
#define HEIGHT 600
#define ANTIALIASING 8

static void setViewport(GLFWwindow *window, int width, int height) {
  glViewport(0, 0, width, height);
  return;
}

void *ts_create_TS_Window() {
  TS_Window *this = (TS_Window *)malloc(sizeof(TS_Window));

  glfwInitHint(GLFW_PLATFORM, GLFW_PLATFORM_X11);
  glfwInit();
  glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
  glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
  glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
  glfwWindowHint(GLFW_SAMPLES, ANTIALIASING);

  this->window = glfwCreateWindow(WIDTH, HEIGHT, "TheSeed", NULL, NULL);

  if (!this->window) {
    printf("Failed to create window");
    exit(1);
  }

  glfwMakeContextCurrent(this->window);

  if (!gladLoadGL(glfwGetProcAddress)) {
    printf("Failed to initialize GLAD");
    exit(1);
  }

  setViewport(this->window, WIDTH, HEIGHT);
  glfwSetFramebufferSizeCallback(this->window, setViewport);

  // Load OpenGL Extensions
  glEnable(GL_DEPTH_TEST);
  glEnable(GL_MULTISAMPLE);

  return this;
}

void ts_destroy_TS_Window(void *w) {
  TS_Window *this = (TS_Window *)w;

  if (this->window) {
    glfwDestroyWindow(this->window);
  }
  free(w);
  return;
}
