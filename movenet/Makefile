SERVER_ENV_FILE="./moveneter_sdk/env"
LINUX_FOLDER := /home/ybyan/Documents/linux
SAMPLE_FOLDER := samples/rust/custom_modules
MODULE_NAME := rust_camera
SDK_MODULE_NAME := rust_sdk
FOLDER_PATH := ${LINUX_FOLDER}/${SAMPLE_FOLDER}
MODULE_FILE_PATH := ${FOLDER_PATH}/${MODULE_NAME}
SDK_MODULE_FILE_PATH := ${FOLDER_PATH}/${SDK_MODULE_NAME}
CORE := 12

MODULE_DEV_FOLDER := ./cam_mod
MODULE_DEV_SRC_FOLDER := src
MODULE_DEV_ARTIFACT_FOLDER := artifact
MODULE_DEV_SRC_PATH := ${MODULE_DEV_FOLDER}/${MODULE_DEV_SRC_FOLDER}
MODULE_DEV_ARTIFACT_PATH := ${MODULE_DEV_FOLDER}/${MODULE_DEV_ARTIFACT_FOLDER}
MODULE_DEV_CAM_MODULE_FILE_PATH := ${MODULE_DEV_SRC_PATH}/${MODULE_NAME}.rs
MODULE_DEV_SDK_MODULE_FILE_PATH := ${MODULE_DEV_SRC_PATH}/${SDK_MODULE_NAME}.rs

build_helper: ${MODULE_DEV_SRC_PATH}/page_helper.c
	cp ${MODULE_DEV_SRC_PATH}/page_helper.c ${FOLDER_PATH}
	(cd ${FOLDER_PATH} && make helper -j${CORE})
	cp ${FOLDER_PATH}/page_helper.ko ${MODULE_DEV_ARTIFACT_PATH}

build_sdk: ${MODULE_DEV_SDK_MODULE_FILE_PATH}
	cp ${MODULE_DEV_SDK_MODULE_FILE_PATH} ${FOLDER_PATH}
	(cd ${FOLDER_PATH} && make sdk -j${CORE})
	cp ${FOLDER_PATH}/${SDK_MODULE_NAME}_wrapper.ko ${MODULE_DEV_ARTIFACT_PATH}

build_cam: ${MODULE_DEV_CAM_MODULE_FILE_PATH}
	cp ${MODULE_DEV_SRC_PATH}/page_helper.c ${FOLDER_PATH}
	cp ${MODULE_DEV_CAM_MODULE_FILE_PATH} ${FOLDER_PATH}
	(cd ${FOLDER_PATH} && make camera -j${CORE})
	cp ${FOLDER_PATH}/page_helper.ko ${MODULE_DEV_ARTIFACT_PATH}
	cp ${FOLDER_PATH}/${MODULE_NAME}.ko ${MODULE_DEV_ARTIFACT_PATH}

build: build_sdk build_cam

upload_mod:
	git add ${MODULE_DEV_ARTIFACT_PATH}
	git commit -m "upload module artifact"
	git push

build_upload_mod: build_cam upload_mod

load_cam: unload_cam unload_sdk load_sdk
	sudo insmod ${MODULE_DEV_ARTIFACT_PATH}/${MODULE_NAME}.ko

unload_cam: 
	sudo rmmod ${MODULE_NAME} || true;

load_sdk: unload_sdk
	sudo insmod ${MODULE_DEV_ARTIFACT_PATH}/${SDK_MODULE_NAME}_wrapper.ko

unload_sdk: 
	sudo rmmod ${SDK_MODULE_NAME}_wrapper || true;

upload_all:
	git add .
	git commit -m "save progress"
	git push

run_server:
	@read -p "Enter the socket address the server should listen to? format: \"{IP_ADDR}:{PORT}\": " LISTEN_ADDR; \
	cargo run -r --bin server -- $$LISTEN_ADDR;
    	
run_client: load_cam
	@read -p "Enter the server address the client should communicate to? format: \"{IP_ADDR}:{PORT}\": " SERVER_ADDR; \
	echo $$SERVER_ADDR > ${SERVER_ENV_FILE}; \
	echo "Written server address $$SERVER_ADDR to file ${SERVER_ENV_FILE}";
	cargo build -r --bin app;
	sudo ./target/release/app;
	
clean:
	cargo clean
