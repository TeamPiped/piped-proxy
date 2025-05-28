#!/usr/bin/env bun

import { program } from "commander";
import cliProgress from "cli-progress";
import { Innertube, UniversalCache } from "youtubei.js";
import { generateWebPoToken } from "./utils.js";

interface SabrTestOptions {
	proxy: string;
	videoId: string;
	duration?: number;
	audioItag?: number;
	videoItag?: number;
	verbose: boolean;
}

interface FormatId {
	itag: number;
	lastModified: number;
	xtags?: string;
}

interface BufferedRange {
	formatId: FormatId;
	startTimeMs: number;
	durationMs: number;
	startSegmentIndex: number;
	endSegmentIndex: number;
}

interface SabrRequestData {
	playerTimeMs: number;
	bandwidthEstimate: number;
	clientViewportWidth: number;
	clientViewportHeight: number;
	playbackRate: number;
	hasAudio: boolean;
	selectedAudioFormatIds: FormatId[];
	selectedVideoFormatIds: FormatId[];
	bufferedRanges: BufferedRange[];
	videoPlaybackUstreamerConfig?: string;
	poToken?: string;
	playbackCookie?: string;
}

class SabrTester {
	private proxyUrl: string;
	private innertube: Innertube | null = null;
	private verbose: boolean;

	constructor(proxyUrl: string, verbose: boolean = false) {
		this.proxyUrl = proxyUrl;
		this.verbose = verbose;
	}

	private log(message: string) {
		if (this.verbose) {
			console.log(`[SABR Test] ${message}`);
		}
	}

	private error(message: string) {
		console.error(`[ERROR] ${message}`);
	}

	async initialize() {
		this.log("Initializing YouTube client...");
		try {
			this.innertube = await Innertube.create({
				cache: new UniversalCache(true),
				enable_session_cache: false,
			});
			this.log("YouTube client initialized successfully");
		} catch (error) {
			throw new Error(`Failed to initialize YouTube client: ${error}`);
		}
	}

	async getVideoInfo(videoId: string) {
		if (!this.innertube) {
			throw new Error("YouTube client not initialized");
		}

		this.log(`Fetching video info for: ${videoId}`);
		try {
			const info = await this.innertube.getBasicInfo(videoId);

			console.log(`
Video Information:
  Title: ${info.basic_info.title}
  Duration: ${info.basic_info.duration}s
  Views: ${info.basic_info.view_count}
  Author: ${info.basic_info.author}
  Video ID: ${info.basic_info.id}
      `);

			return info;
		} catch (error) {
			throw new Error(`Failed to get video info: ${error}`);
		}
	}

	async generatePoToken(): Promise<string | undefined> {
		if (!this.innertube) {
			throw new Error("YouTube client not initialized");
		}

		try {
			this.log("Generating PoToken...");
			const visitorData = this.innertube.session.context.client.visitorData;
			if (!visitorData) {
				this.log("No visitor data available, skipping PoToken generation");
				return undefined;
			}

			const webPoTokenResult = await generateWebPoToken(visitorData);
			this.log(`PoToken generated successfully: ${webPoTokenResult.poToken.substring(0, 20)}...`);
			return webPoTokenResult.poToken;
		} catch (error) {
			this.log(`PoToken generation failed: ${error}`);
			return undefined;
		}
	}

	async testSabrRequest(videoId: string, options: Partial<SabrTestOptions> = {}) {
		if (!this.innertube) {
			throw new Error("YouTube client not initialized");
		}

		const info = await this.getVideoInfo(videoId);

		// Get streaming data
		const streamingData = info.streaming_data;
		if (!streamingData) {
			throw new Error("No streaming data available");
		}

		// Get server ABR streaming URL
		const serverAbrStreamingUrl = this.innertube.session.player?.decipher(streamingData.server_abr_streaming_url);

		if (!serverAbrStreamingUrl) {
			throw new Error("No server ABR streaming URL found");
		}

		this.log(`Server ABR URL: ${serverAbrStreamingUrl}`);

		// Get video playback ustreamer config
		const videoPlaybackUstreamerConfig =
			info.page?.[0]?.player_config?.media_common_config?.media_ustreamer_request_config
				?.video_playback_ustreamer_config;

		if (!videoPlaybackUstreamerConfig) {
			throw new Error("No video playback ustreamer config found");
		}

		// Generate PoToken
		const poToken = await this.generatePoToken();

		// Find suitable formats
		const audioFormat = streamingData.adaptive_formats.find(
			(f: any) =>
				f.mime_type?.includes("audio") && (options.audioItag ? f.itag === options.audioItag : f.itag === 251),
		);

		const videoFormat = streamingData.adaptive_formats.find(
			(f: any) =>
				f.mime_type?.includes("video") && (options.videoItag ? f.itag === options.videoItag : f.itag === 136),
		);

		let finalAudioFormat, finalVideoFormat;

		if (!audioFormat || !videoFormat) {
			// If specific formats not found, try to find any audio/video formats
			const fallbackAudio = streamingData.adaptive_formats.find((f: any) => f.mime_type?.includes("audio"));
			const fallbackVideo = streamingData.adaptive_formats.find((f: any) => f.mime_type?.includes("video"));

			if (!fallbackAudio || !fallbackVideo) {
				throw new Error("Could not find suitable audio/video formats");
			}

			console.log(`
Selected Formats (fallback):
  Audio: itag=${fallbackAudio.itag}, mime=${fallbackAudio.mime_type}
  Video: itag=${fallbackVideo.itag}, mime=${fallbackVideo.mime_type}
      `);

			finalAudioFormat = fallbackAudio;
			finalVideoFormat = fallbackVideo;
		} else {
			console.log(`
Selected Formats:
  Audio: itag=${audioFormat.itag}, mime=${audioFormat.mime_type}
  Video: itag=${videoFormat.itag}, mime=${videoFormat.mime_type}
      `);

			finalAudioFormat = audioFormat;
			finalVideoFormat = videoFormat;
		}

		// Prepare SABR request data with correct structure matching Rust handler
		const sabrData: SabrRequestData = {
			playerTimeMs: 0,
			bandwidthEstimate: 1000000, // 1 Mbps
			clientViewportWidth: 1920,
			clientViewportHeight: 1080,
			playbackRate: 1.0,
			hasAudio: true,
			selectedAudioFormatIds: [
				{
					itag: finalAudioFormat.itag!,
					lastModified: parseInt(finalAudioFormat.last_modified_ms || "0"),
					xtags: finalAudioFormat.xtags,
				},
			],
			selectedVideoFormatIds: [
				{
					itag: finalVideoFormat.itag!,
					lastModified: parseInt(finalVideoFormat.last_modified_ms || "0"),
					xtags: finalVideoFormat.xtags,
				},
			],
			bufferedRanges: [], // Empty for initial request
			videoPlaybackUstreamerConfig: Buffer.from(videoPlaybackUstreamerConfig).toString("base64"),
			poToken: poToken ? Buffer.from(poToken, "utf-8").toString("base64") : undefined,
		};

		// Test SABR request through proxy
		await this.testProxySabrRequest(serverAbrStreamingUrl, sabrData, options.duration || 10);
	}

	async testProxySabrRequest(serverAbrUrl: string, sabrData: SabrRequestData, durationSeconds: number) {
		const url = new URL(serverAbrUrl);

		// Add sabr parameter to indicate this is a SABR request
		url.searchParams.set("sabr", "1");

		// Replace the host with our proxy
		const proxyUrl = new URL(this.proxyUrl);
		url.searchParams.set("host", url.host);

		const finalUrl = `${proxyUrl.origin}/videoplayback?${url.searchParams.toString()}`;

		this.log(`Making SABR request to proxy: ${finalUrl}`);

		const progressBar = new cliProgress.SingleBar({
			format: "SABR Test [{bar}] {percentage}% | ETA: {eta}s | {value}/{total}s",
			barCompleteChar: "\u2588",
			barIncompleteChar: "\u2591",
			hideCursor: true,
		});

		progressBar.start(durationSeconds, 0);

		try {
			const response = await fetch(finalUrl, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
					"User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
				},
				body: JSON.stringify(sabrData),
			});

			if (!response.ok) {
				throw new Error(`SABR request failed: ${response.status} ${response.statusText}`);
			}

			this.log(`SABR request successful: ${response.status}`);
			this.log(`Response headers: ${JSON.stringify(Object.fromEntries(response.headers.entries()), null, 2)}`);

			// Check for playback cookie in response headers
			const playbackCookie = response.headers.get("X-Playback-Cookie");
			if (playbackCookie) {
				this.log(`Received playback cookie: ${playbackCookie.substring(0, 50)}...`);
			}

			// Read response body (this would be the UMP stream)
			const reader = response.body?.getReader();
			if (!reader) {
				throw new Error("No response body reader available");
			}

			let totalBytes = 0;
			let chunks = 0;
			const startTime = Date.now();

			while (true) {
				const { done, value } = await reader.read();
				if (done) break;

				totalBytes += value.length;
				chunks++;

				const elapsedSeconds = (Date.now() - startTime) / 1000;
				progressBar.update(Math.min(elapsedSeconds, durationSeconds));

				if (elapsedSeconds >= durationSeconds) {
					this.log("Duration limit reached, stopping...");
					break;
				}

				// Log first few chunks for debugging
				if (chunks <= 3 && this.verbose) {
					this.log(`Chunk ${chunks}: ${value.length} bytes`);
					this.log(
						`First 32 bytes: ${Array.from(value.slice(0, 32))
							.map((b) => (b as number).toString(16).padStart(2, "0"))
							.join(" ")}`,
					);
				}
			}

			progressBar.stop();

			console.log(`
SABR Test Results:
  Total bytes received: ${totalBytes}
  Total chunks: ${chunks}
  Average chunk size: ${Math.round(totalBytes / chunks)} bytes
  Duration: ${((Date.now() - startTime) / 1000).toFixed(2)}s
  Throughput: ${(totalBytes / 1024 / 1024 / ((Date.now() - startTime) / 1000)).toFixed(2)} MB/s
      `);

			return true;
		} catch (error) {
			progressBar.stop();
			throw error;
		}
	}
}

// CLI setup
program.name("sabr-test").description("Test SABR functionality through piped-proxy").version("1.0.0");

program
	.option("-p, --proxy <url>", "Proxy server URL", "http://127.0.0.1:8080")
	.option("-v, --video-id <id>", "YouTube video ID to test with", "eg2g6FPsdLI")
	.option("-d, --duration <seconds>", "Test duration in seconds", "10")
	.option("-a, --audio-itag <itag>", "Audio format itag to use")
	.option("--video-itag <itag>", "Video format itag to use")
	.option("--verbose", "Enable verbose logging", false);

program.action(async (options: any) => {
	const tester = new SabrTester(options.proxy, options.verbose);

	try {
		console.log(`
🚀 SABR Proxy Tester
Proxy: ${options.proxy}
Video ID: ${options.videoId}
Duration: ${options.duration}s
    `);

		await tester.initialize();

		// Test SABR functionality
		console.log("🔄 Testing SABR functionality...");
		await tester.testSabrRequest(options.videoId, {
			duration: parseInt(options.duration),
			audioItag: options.audioItag ? parseInt(options.audioItag) : undefined,
			videoItag: options.videoItag ? parseInt(options.videoItag) : undefined,
			verbose: options.verbose,
			proxy: options.proxy,
			videoId: options.videoId,
		});

		console.log("✅ SABR test completed successfully!");
	} catch (error) {
		console.error(`❌ Test failed: ${error}`);
		process.exit(1);
	}
});

// Parse command line arguments
program.parse();
