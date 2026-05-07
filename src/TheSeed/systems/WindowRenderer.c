#include "TheSeed/ecs/Scene.h"
#include "glad/gl.h"

#include "TheSeed/components/Window.h"
#include "TheSeed/ecs/logging.h"
#include "TheSeed/systems/WindowRenderer.h"
#include <GLFW/glfw3.h>
#include <stdio.h>
#include <stdlib.h>

TS_System_Groups ts_groups_ts_window_pre_renderer = 1;

TS_System_Groups ts_select_ts_window_pre_renderer(const TS_Scene *scene,
                                                  const TS_Entity entity) {
  if (ts_entity_get_component(scene, entity, "TS_Window")) {
    return 1;
  }
  return 0;
}

void ts_system_ts_window_pre_renderer(TS_Scene *scene, TS_Entity **entities,
                                      const size_t *groups) {
  for (size_t i = 0; i < *groups; i++) {
    TS_Window *window = (TS_Window *)ts_entity_get_component(
        scene, entities[0][i], "TS_Window");

    GLFWwindow *glfw_window = (GLFWwindow *)ts_get(
        scene, window,
        "window"); // Discard const qualifier because direct access is needed
    if (!glfw_window) {
      ts_warn("Window attribute is NULL");
      continue;
    }
    if (!glfwWindowShouldClose(glfw_window)) {
      glClearColor(0.1F, 0.1F, 0.1F, 1.0F);
      glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    } else {
      ts_remove_component(scene, entities[0][i], "TS_Window");
    }
  }
}

TS_System_Groups ts_groups_ts_window_post_renderer = 1;

TS_System_Groups ts_select_ts_window_post_renderer(const TS_Scene *scene,
                                                   const TS_Entity entity) {
  if (ts_entity_get_component(scene, entity, "TS_Window")) {
    return 1;
  }
  return 0;
}

void ts_system_ts_window_post_renderer(TS_Scene *scene, TS_Entity **entities,
                                       const size_t *groups) {
  for (size_t i = 0; i < *groups; i++) {
    TS_Window *window = (TS_Window *)ts_entity_get_component(
        scene, entities[0][i], "TS_Window");

    GLFWwindow *glfw_window = (GLFWwindow *)ts_get(
        scene, window,
        "window"); // Discard const qualifier because direct access is needed
    if (!glfw_window) {
      ts_warn("Window attribute is NULL");
      continue;
    }
    if (!glfwWindowShouldClose(glfw_window)) {
      glfwSwapBuffers(glfw_window);
      glfwPollEvents();
    } else {
      ts_remove_component(scene, entities[0][i], "TS_Window");
    }
  }
}

TS_System_Groups ts_groups_ts_window_quiter = 1;

TS_System_Groups ts_select_ts_window_quiter(const TS_Scene *scene,
                                            const TS_Entity entity) {
  if (ts_entity_get_component(scene, entity, "TS_Window")) {
    return 1;
  }
  return 0;
}

void ts_system_ts_window_quiter(TS_Scene *scene, TS_Entity **entities,
                                const size_t *groups) {
  if (*groups == 0) {
    ts_set_scene_terminate(scene);
  }
}

TS_System_Groups ts_groups_ts_window_reloader = 1;

TS_System_Groups ts_select_ts_window_reloader(const TS_Scene *scene,
                                              const TS_Entity entity) {
  if (ts_entity_get_component(scene, entity, "TS_Window")) {
    return 1;
  }
  return 0;
}

void ts_system_ts_window_reloader(TS_Scene *scene, TS_Entity **entities,
                                  const size_t *groups) {
  for (size_t i = 0; i < *groups; i++) {
    TS_Window *window = (TS_Window *)ts_entity_get_component(
        scene, entities[0][i], "TS_Window");

    GLFWwindow *glfw_window = (GLFWwindow *)ts_get(
        scene, window,
        "window"); // Discard const qualifier because direct access is needed
    if (!glfw_window) {
      ts_warn("Window attribute is NULL");
      continue;
    }
    if (glfwGetKey(glfw_window, GLFW_KEY_R) == GLFW_PRESS) {
      ts_set_scene_reload(scene);
      return;
    }
  }
}
