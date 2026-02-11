#include "glad/gl.h"

#include "TheSeed/components/Window.h"
#include "TheSeed/systems/WindowRenderer.h"
#include <GLFW/glfw3.h>
#include <stdio.h>
#include <stdlib.h>

int ts_select_ts_window_renderer(TS_Scene_t *scene, const size_t entity) {
  if (ts_entity_get_component(scene, entity, "TS_Window")) {
    return 1;
  } else {
    return 0;
  }
}

void ts_system_ts_window_renderer(TS_Scene_t *scene, size_t *entities,
                                  size_t n) {
  for (size_t i = 0; i < n; i++) {
    TS_Window *window =
        (TS_Window *)ts_entity_get_component(scene, entities[i], "TS_Window");

    if (!glfwWindowShouldClose(window->window)) {
      glfwSwapBuffers(window->window);
      glfwPollEvents();
      glClearColor(0.1f, 0.1f, 0.1f, 1.0f);
      glClear(GL_COLOR_BUFFER_BIT);
    } else {
      ts_remove_component(scene, entities[i], "TS_Window");
    }
  }
}

int ts_select_ts_window_quiter(TS_Scene_t *scene, const size_t entity) {
  if (ts_entity_get_component(scene, entity, "TS_Window")) {
    return 1;
  } else {
    return 0;
  }
}

void ts_system_ts_window_quiter(TS_Scene_t *scene, size_t *entities, size_t n) {
  if (n == 0) {
    glfwTerminate();
    exit(0);
  }
}

int ts_select_ts_window_reloader(TS_Scene_t *scene, const size_t entity) {
  if (ts_entity_get_component(scene, entity, "TS_Window")) {
    return 1;
  } else {
    return 0;
  }
}

void ts_system_ts_window_reloader(TS_Scene_t *scene, size_t *entities,
                                  size_t n) {
  for (size_t i = 0; i < n; i++) {
    TS_Window *window =
        (TS_Window *)ts_entity_get_component(scene, entities[i], "TS_Window");

    if (glfwGetKey(window->window, GLFW_KEY_R) == GLFW_PRESS) {
      printf("Beginning Reload!\n");
      // int status = ts_reload_plugin(scene, "build/libtheseed_systems.so",
      //                               "build/libtheseed_systems.so");
      int status = ts_reload_all_plugins(scene);
      printf("Status after reload: %d\n", status);
    }
  }
}
