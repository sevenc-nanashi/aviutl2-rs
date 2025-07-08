//----------------------------------------------------------------------------------
//	サンプルAVI(vfw経由)入力プラグイン for AviUtl ExEdit2
//----------------------------------------------------------------------------------
#include <windows.h>
#include <vfw.h>
#pragma comment(lib, "vfw32.lib")

#include "input2.h"

void func_init(void);
void func_exit(void);
INPUT_HANDLE func_open(LPCWSTR file);
bool func_close(INPUT_HANDLE ih);
bool func_info_get(INPUT_HANDLE ih, INPUT_INFO* iip);
int func_read_video(INPUT_HANDLE ih, int frame, void* buf);
int func_read_audio(INPUT_HANDLE ih, int start, int length, void* buf);
bool func_config(HWND hwnd, HINSTANCE dll_hinst);

//---------------------------------------------------------------------
//	入力プラグイン構造体定義
//---------------------------------------------------------------------
INPUT_PLUGIN_TABLE input_plugin_table = {
	INPUT_PLUGIN_TABLE::FLAG_VIDEO | INPUT_PLUGIN_TABLE::FLAG_AUDIO, //	フラグ
	L"AVI File Reader2 (sample)",						// プラグインの名前
	L"AviFile (*.avi)¥0*.avi¥0",						// 入力ファイルフィルタ
	L"AVI File Reader2 version 2.00 By ＫＥＮくん",		// プラグインの情報
	func_open,											// 入力ファイルをオープンする関数へのポインタ
	func_close,											// 入力ファイルをクローズする関数へのポインタ
	func_info_get,										// 入力ファイルの情報を取得する関数へのポインタ
	func_read_video,									// 画像データを読み込む関数へのポインタ
	func_read_audio,									// 音声データを読み込む関数へのポインタ
	func_config,										// 入力設定のダイアログを要求された時に呼ばれる関数へのポインタ (nullptrなら呼ばれません)
};

//---------------------------------------------------------------------
//	ファイルハンドル構造体
//---------------------------------------------------------------------
struct FILE_HANDLE {
	int				flag;
	static constexpr int FLAG_VIDEO = 1;
	static constexpr int FLAG_AUDIO = 2;
	PAVIFILE		pfile;
	PAVISTREAM		pvideo, paudio;
	AVIFILEINFO		fileinfo;
	AVISTREAMINFO	videoinfo, audioinfo;
	void*			videoformat;
	LONG			videoformatsize;
	void*			audioformat;
	LONG			audioformatsize;
};

//---------------------------------------------------------------------
//	入力プラグイン構造体のポインタを渡す関数
//---------------------------------------------------------------------
EXTERN_C __declspec(dllexport) INPUT_PLUGIN_TABLE* GetInputPluginTable(void)
{
	return &input_plugin_table;
}

//---------------------------------------------------------------------
//	DLLエントリポイント
//---------------------------------------------------------------------
BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved) {
	switch (fdwReason) {
		case DLL_PROCESS_ATTACH:
			func_init();
			break;
		case DLL_PROCESS_DETACH:
			if (lpvReserved != nullptr) break;
			func_exit();
			break;
	}
	return TRUE;
}

//---------------------------------------------------------------------
//	初期化
//---------------------------------------------------------------------
void func_init( void ) {
	AVIFileInit();
}

//---------------------------------------------------------------------
//	終了
//---------------------------------------------------------------------
void func_exit( void ) {
	AVIFileExit();
}

//---------------------------------------------------------------------
//	ファイルオープン
//---------------------------------------------------------------------
INPUT_HANDLE func_open(LPCWSTR file) {
	FILE_HANDLE* fp = (FILE_HANDLE*)GlobalAlloc(GMEM_FIXED, sizeof(FILE_HANDLE));
	if (fp == NULL) return NULL;
	ZeroMemory(fp, sizeof(FILE_HANDLE));

	if (AVIFileOpen(&fp->pfile, file, OF_READ, nullptr) != S_OK) {
		GlobalFree(fp);
		return nullptr;
	}

	if (AVIFileInfo(fp->pfile, &fp->fileinfo, sizeof(fp->fileinfo)) == S_OK) {
		for (DWORD i = 0; i < fp->fileinfo.dwStreams; i++) {
			PAVISTREAM pas;
			if (AVIFileGetStream(fp->pfile, &pas, 0, i) == S_OK) {
				AVISTREAMINFO asi;
				AVIStreamInfo(pas, &asi, sizeof(asi));
				if (asi.fccType == streamtypeVIDEO) {
					//	ビデオストリームの設定
					fp->pvideo = pas;
					fp->videoinfo = asi;
					AVIStreamFormatSize(fp->pvideo, 0, &fp->videoformatsize);
					fp->videoformat = (FILE_HANDLE*)GlobalAlloc(GMEM_FIXED, fp->videoformatsize);
					if (fp->videoformat) {
						AVIStreamReadFormat(fp->pvideo, 0, fp->videoformat, &fp->videoformatsize);
						fp->flag |= FILE_HANDLE::FLAG_VIDEO;
					} else {
						AVIStreamRelease(pas);
					}
				} else if (asi.fccType == streamtypeAUDIO) {
					//	オーディオストリームの設定
					fp->paudio = pas;
					fp->audioinfo = asi;
					AVIStreamFormatSize(fp->paudio, 0, &fp->audioformatsize);
					fp->audioformat = (FILE_HANDLE*)GlobalAlloc(GMEM_FIXED, fp->audioformatsize);
					if (fp->videoformat) {
						AVIStreamReadFormat(fp->paudio, 0, fp->audioformat, &fp->audioformatsize);
						fp->flag |= FILE_HANDLE::FLAG_AUDIO;
					} else {
						AVIStreamRelease(pas);
					}
				} else {
					AVIStreamRelease(pas);
				}
			}
		}
	}

	return fp;
}

//---------------------------------------------------------------------
//	ファイルクローズ
//---------------------------------------------------------------------
bool func_close(INPUT_HANDLE ih) {
	FILE_HANDLE* fp = (FILE_HANDLE*)ih;

	if (fp) {
		if (fp->flag & FILE_HANDLE::FLAG_AUDIO) {
			AVIStreamRelease(fp->paudio);
			GlobalFree(fp->audioformat);
		}
		if (fp->flag & FILE_HANDLE::FLAG_VIDEO) {
			AVIStreamRelease(fp->pvideo);
			GlobalFree(fp->videoformat);
		}
		AVIFileRelease(fp->pfile);
		GlobalFree(fp);
	}

	return true;
}

//---------------------------------------------------------------------
//	ファイルの情報
//---------------------------------------------------------------------
bool func_info_get(INPUT_HANDLE ih, INPUT_INFO* iip) {
	FILE_HANDLE	*fp = (FILE_HANDLE *)ih;

	iip->flag = 0;
	if (fp->flag & FILE_HANDLE::FLAG_VIDEO) {
		iip->flag |= INPUT_INFO::FLAG_VIDEO;
		iip->rate = fp->videoinfo.dwRate;
		iip->scale = fp->videoinfo.dwScale;
		iip->n = fp->videoinfo.dwLength;
		iip->format = (BITMAPINFOHEADER*)fp->videoformat;
		iip->format_size = fp->videoformatsize;
	}

	if (fp->flag & FILE_HANDLE::FLAG_AUDIO) {
		iip->flag |= INPUT_INFO::FLAG_AUDIO;
		iip->audio_n = fp->audioinfo.dwLength;
		iip->audio_format = (WAVEFORMATEX*)fp->audioformat;
		iip->audio_format_size = fp->audioformatsize;
	}

	return true;
}

//---------------------------------------------------------------------
//	画像読み込み
//---------------------------------------------------------------------
int func_read_video(INPUT_HANDLE ih, int frame, void* buf) {
	FILE_HANDLE* fp = (FILE_HANDLE*)ih;

	LONG videosize, size;
	if (AVIStreamRead(fp->pvideo, frame, 1, NULL, NULL, &videosize, NULL) != S_OK) return 0;
	if (AVIStreamRead(fp->pvideo, frame, 1, buf, videosize, &size, NULL) != S_OK) return 0;
	return size;
}

//---------------------------------------------------------------------
//	音声読み込み
//---------------------------------------------------------------------
int func_read_audio(INPUT_HANDLE ih, int start, int length, void* buf) {
	FILE_HANDLE* fp = (FILE_HANDLE*)ih;
	LONG size;
	int samplesize;

	samplesize = ((WAVEFORMATEX*)fp->audioformat)->nBlockAlign;
	if (AVIStreamRead(fp->paudio, start, length, buf, samplesize * length, NULL, &size) != S_OK) return 0;
	return size;
}

//---------------------------------------------------------------------
//	設定ダイアログ
//---------------------------------------------------------------------
bool func_config(HWND hwnd, HINSTANCE dll_hinst) {
	MessageBox(hwnd, L"サンプルダイアログ", L"入力設定", MB_OK);

	// DLLを開放されても設定が残るように保存しておいてください

	return true;
}
