#include <glad/gl.h>

#include "GL/gl.h"
#include "TheSeed/core/Window.h"
#include "TheSeed/ecs/Macros.h"
#include "TheSeed/ecs/Scene.h"
#include "TheSeed/ecs/logging.h"
#include <GLFW/glfw3.h>
#include <stdio.h>
#include <stdlib.h>

#define WIDTH 800
#define HEIGHT 600
#define ANTIALIASING 8

// LSAN fixes
#ifdef __has_feature
#if __has_feature(address_sanitizer) || __has_feature(leak_sanitizer)
#define HAVE_LSAN 1
#endif
#endif

#ifndef HAVE_LSAN
#if defined(__SANITIZE_ADDRESS__) || defined(__SANITIZE_LEAK__)
#define HAVE_LSAN 1
#endif
#endif

#ifdef HAVE_LSAN
#include <sanitizer/lsan_interface.h>
#define LSAN_DISABLE() __lsan_disable()
#define LSAN_ENABLE() __lsan_enable()
#else
#define LSAN_DISABLE() ((void)0)
#define LSAN_ENABLE() ((void)0)
#endif

struct TS_Window {
  GLFWwindow *window;
};

static void setViewport(GLFWwindow *window, int width, int height) {
  glViewport(0, 0, width, height);
}

void *ts_create_TS_Window() {
  TS_Window *this = (TS_Window *)malloc(sizeof(TS_Window));
  ts_assert(this, "Malloc failed during ts_create_TS_Window");

  glfwInitHint(GLFW_PLATFORM, GLFW_PLATFORM_X11);
  glfwInit();
  glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
  glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
  glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
  glfwWindowHint(GLFW_CONTEXT_CREATION_API, GLFW_EGL_CONTEXT_API);
  glfwWindowHint(GLFW_SAMPLES, ANTIALIASING);

  LSAN_DISABLE();
  this->window = glfwCreateWindow(WIDTH, HEIGHT, "TheSeed", NULL, NULL);
  LSAN_ENABLE();

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

void ts_destroy_TS_Window(void *window) {
  TS_Window *this = (TS_Window *)window;

  if (this->window) {
    glfwDestroyWindow(this->window);
    glfwTerminate();
  }
  free(window);
}

TS_BASIC_GETTER(TS_Window, window, component->window, sizeof(GLFWwindow *));

void ts_schema_TS_Window(TS_Component_Schema *schema) {
  TS_SCHEMA_FIELD(TS_BLOB, window, ts_get_TS_Window_window, NULL, NULL, NULL);
}
