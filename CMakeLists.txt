cmake_minimum_required(VERSION 3.15)
project(NailSnake
  VERSION 1.0.0
  DESCRIPTION "Cross-platform terminal Snake for Windows, Linux, and macOS"
  LANGUAGES NONE
)

# NailSnake is a Rust project built with Cargo.
# This CMake file provides convenience targets for IDEs and users who
# want to integrate the build into CMake-based workflows.

set(BIN_NAME "nailsnake")
set(MAN_SRC "${CMAKE_CURRENT_SOURCE_DIR}/man/${BIN_NAME}.1")

# --- Custom targets ---

add_custom_target(build ALL
  COMMAND cargo build --release
  COMMENT "Building NailSnake release binary"
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_target(debug
  COMMAND cargo build
  COMMENT "Building NailSnake debug binary"
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_target(check
  COMMAND cargo check
  COMMENT "Running cargo check"
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_target(test
  COMMAND cargo test
  COMMENT "Running tests"
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_target(clean
  COMMAND cargo clean
  COMMENT "Cleaning build artifacts"
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_target(run
  COMMAND cargo run
  COMMENT "Running NailSnake via cargo"
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

# --- Install rules ---

set(BIN_DEST "${CMAKE_INSTALL_BINDIR}" CACHE PATH "Binary install directory")
if(NOT BIN_DEST)
  if(WIN32)
    set(BIN_DEST "bin")
  else()
    set(BIN_DEST "${CMAKE_INSTALL_PREFIX}/bin")
  endif()
endif()

set(MAN1_DEST "${CMAKE_INSTALL_MANDIR}/man1" CACHE PATH "Man page install directory")
if(NOT MAN1_DEST)
  if(NOT WIN32)
    set(MAN1_DEST "${CMAKE_INSTALL_PREFIX}/share/man/man1")
  endif()
endif()

install(
  FILES "${CMAKE_CURRENT_SOURCE_DIR}/target/release/${BIN_NAME}$<$<PLATFORM_ID:Windows>:.exe>"
  DESTINATION "${BIN_DEST}"
  PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE GROUP_READ GROUP_EXECUTE WORLD_READ WORLD_EXECUTE
)

if(MAN1_DEST)
  install(
    FILES "${MAN_SRC}"
    DESTINATION "${MAN1_DEST}"
    PERMISSIONS OWNER_READ GROUP_READ WORLD_READ
  )

  get_filename_component(MAN_DIR "${MAN1_DEST}" DIRECTORY)
  install(CODE "
    find_program(MANDB mandb QUIET)
    if(MANDB)
      execute_process(COMMAND \${MANDB} -q \"${MAN_DIR}\"
        COMMAND_ERROR_IS_FATAL LAST
      )
      message(STATUS \"man-db cache updated\")
    elseif(CMAKE_HOST_APPLE)
      find_program(MAKEWHATIS makewhatis QUIET)
      if(MAKEWHATIS)
        execute_process(COMMAND \${MAKEWHATIS} \"${MAN1_DEST}\"
          COMMAND_ERROR_IS_FATAL LAST
        )
        message(STATUS \"makewhatis cache updated\")
      endif()
    endif()
  ")
endif()

# --- Packaging ---

set(CPACK_PACKAGE_NAME "NailSnake")
set(CPACK_PACKAGE_VERSION "${PROJECT_VERSION}")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "${PROJECT_DESCRIPTION}")
set(CPACK_PACKAGE_VENDOR "voltsparx")
set(CPACK_PACKAGE_CONTACT "voltsparx@gmail.com")
set(CPACK_PACKAGE_FILE_NAME "${BIN_NAME}-${PROJECT_VERSION}")
set(CPACK_RESOURCE_FILE_LICENSE "${CMAKE_CURRENT_SOURCE_DIR}/LICENSE")
set(CPACK_RESOURCE_FILE_README "${CMAKE_CURRENT_SOURCE_DIR}/README.md")

if(WIN32)
  set(CPACK_GENERATOR "NSIS")
elseif(APPLE)
  set(CPACK_GENERATOR "DragNDrop")
else()
  set(CPACK_GENERATOR "TGZ;DEB")
  set(CPACK_DEBIAN_PACKAGE_MAINTAINER "voltsparx <voltsparx@gmail.com>")
  set(CPACK_DEBIAN_FILE_NAME DEB-DEFAULT)
endif()

include(CPack)
