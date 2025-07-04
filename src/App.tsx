import "./App.css";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Separator } from "@/components/ui/separator";
import { Download, Settings, History, CheckCircle, AlertCircle, FolderOpen, Trash2, ExternalLink } from "lucide-react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useState, useEffect } from "react";
import { TauriYouTubeAPI, DownloadRequest, AppSettings } from "./lib/tauri-api";
import { open } from '@tauri-apps/plugin-dialog';
import { openUrl } from '@tauri-apps/plugin-opener';
import { ask, message } from '@tauri-apps/plugin-dialog';
import { ScrollArea } from "@/components/ui/scroll-area";

function App() {
  const [url, setUrl] = useState("");
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadHistory, setDownloadHistory] = useState([
    { id: "1", title: "Sample Video Title", url: "https://youtube.com/watch?v=example", status: "completed", date: "2024-01-15" }
  ]);
  const [outputPath, setOutputPath] = useState("");
  const [selectedFormat, setSelectedFormat] = useState("mp3");
  const [selectedQuality, setSelectedQuality] = useState("high");
  const [isClearingHistory, setIsClearingHistory] = useState(false);

  useEffect(() => {
    loadSettings();
    loadDownloadHistory();
  }, []);

  const loadDefaultPath = async () => {
    try {
      const defaultPath = await TauriYouTubeAPI.getDefaultDownloadPath();
      setOutputPath(defaultPath);
    } catch (error) {
      console.error("Error getting default path:", error);
    }
  };

  const loadSettings = async () => {
    try {
      const settings = await TauriYouTubeAPI.loadSettings();
      setOutputPath(settings.download_path);
      setSelectedFormat(settings.default_format);
      setSelectedQuality(settings.default_quality);
      console.log("Settings loaded:", settings);
    } catch (error) {
      console.error("Error loading settings:", error);
      await loadDefaultPath();
    }
  };

  const selectDownloadFolder = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: true,
      });
      
      if (selected) {
        setOutputPath(selected as string);
        console.log("Selected folder:", selected);
        
        const newSettings: AppSettings = {
          download_path: selected as string,
          default_format: selectedFormat,
          default_quality: selectedQuality,
        };
        await TauriYouTubeAPI.saveSettings(newSettings);
        console.log("Settings auto-saved after folder selection");
      }
    } catch (error) {
      console.error("Error opening folder dialog:", error);
      alert("Error opening folder selection dialog");
    }
  };

  const openInBrowser = async (url: string) => {
    try {
      await openUrl(url);
    } catch (error) {
      console.error('Error opening URL:', error);
      try {
        await navigator.clipboard.writeText(url);
        await message('URL copied to clipboard', {
          title: 'Cannot open browser',
          kind: 'info'
        });
      } catch (clipboardError) {
        console.error('Error copying to clipboard:', clipboardError);
        await message('Failed to open URL and copy to clipboard', {
          title: 'Error',
          kind: 'error'
        });
      }
    }
  };

  const handleDownload = async () => {
    if (!url.trim()) return;
    
    try {
      const isValid = await TauriYouTubeAPI.validateUrl(url);
      if (!isValid) {
        alert("Invalid YouTube URL!");
        return;
      }

      setIsDownloading(true);
      
      const videoInfo = await TauriYouTubeAPI.getVideoInfo(url);
      console.log("Video info:", videoInfo);

      const downloadRequest: DownloadRequest = {
        url: url,
        format: selectedFormat,
        quality: selectedQuality,
        output_path: outputPath,
        filename: undefined,
      };

      const downloadId = await TauriYouTubeAPI.downloadVideo(downloadRequest);
      console.log("Download started with ID:", downloadId);

      const checkDownloadStatus = async () => {
        try {
          const status = await TauriYouTubeAPI.getDownloadStatus(downloadId);
          if (status) {
            console.log("Download status:", status);
            
            if (status.status === 'Completed') {
              setDownloadProgress(100);
              setIsDownloading(false);
              loadDownloadHistory();
              setUrl("");
              return true;
            } else if (status.status === 'Failed') {
              setIsDownloading(false);
              setDownloadProgress(0);
              loadDownloadHistory();
              await message(`Download failed`, {
                title: "Download Error",
                kind: "error"
              });
              return true;
            } else if (status.status === 'Downloading') {
              setDownloadProgress(prev => Math.min(prev + 10, 90));
            }
          }
        } catch (error) {
          console.error("Error checking download status:", error);
        }
        return false;
      };

      const interval = setInterval(async () => {
        const finished = await checkDownloadStatus();
        if (finished) {
          clearInterval(interval);
        }
      }, 2000);

      setTimeout(() => {
        clearInterval(interval);
        if (isDownloading) {
          setIsDownloading(false);
          setDownloadProgress(0);
          loadDownloadHistory();
        }
      }, 300000);

    } catch (error) {
      console.error("Download failed:", error);
      setIsDownloading(false);
      await message(`Download error: ${error}`, {
        title: "Error",
        kind: "error"
      });
    }
  };

  const loadDownloadHistory = async () => {
    try {
      const history = await TauriYouTubeAPI.getDownloadHistory();
      console.log("Download history:", history);
      
      const uiHistory = history.map(item => ({
        id: item.id,
        title: item.title,
        url: item.url,
        status: item.status === 'Completed' ? 'completed' : 'failed',
        date: new Date(item.downloaded_at).toISOString().split('T')[0]
      }));
      
      setDownloadHistory(uiHistory);
    } catch (error) {
      console.error("Error loading history:", error);
    }
  };

  const clearHistory = async () => {
    if (isClearingHistory) return;
    
    try {
      setIsClearingHistory(true);
      
      const confirmed = await ask("Are you sure you want to delete the entire download history?", {
        title: "Confirm Deletion",
        kind: "warning"
      });
      
      if (!confirmed) {
        setIsClearingHistory(false);
        return;
      }

      await TauriYouTubeAPI.clearDownloadHistory();
      setDownloadHistory([]);
      console.log("Download history cleared");
    } catch (error) {
      console.error("Error clearing history:", error);
    } finally {
      setIsClearingHistory(false);
    }
  };

  const testBackend = async () => {
    console.log("=== Testing Backend ===");
    
    try {
      console.log("1. Checking dependencies...");
      const deps = await TauriYouTubeAPI.checkDependencies();
      console.log("Dependencies OK:", deps);

      console.log("2. Getting supported formats...");
      const formats = await TauriYouTubeAPI.getSupportedFormats();
      console.log("Formats:", formats);

      console.log("3. Getting supported qualities...");
      const qualities = await TauriYouTubeAPI.getSupportedQualities();
      console.log("Qualities:", qualities);

      console.log("4. Testing URL validation...");
      const validUrl = await TauriYouTubeAPI.validateUrl("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
      const invalidUrl = await TauriYouTubeAPI.validateUrl("https://example.com");
      console.log("Valid YouTube URL:", validUrl);
      console.log("Invalid URL:", invalidUrl);

      console.log("5. Getting video info (Rick Roll)...");
      const videoInfo = await TauriYouTubeAPI.getVideoInfo("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
      console.log("Video info:", videoInfo);

      console.log("=== Backend Test Complete ===");
    } catch (error) {
      console.error("Backend test failed:", error);
    }
  };

  useEffect(() => {
    (window as any).testBackend = testBackend;
    console.log("Backend test function available as window.testBackend()");
  }, []);

  useEffect(() => {
    if (outputPath && selectedFormat && selectedQuality) {
      const autoSave = async () => {
        try {
          const settings: AppSettings = {
            download_path: outputPath,
            default_format: selectedFormat,
            default_quality: selectedQuality,
          };
          await TauriYouTubeAPI.saveSettings(settings);
          console.log("Settings auto-saved:", settings);
        } catch (error) {
          console.error("Error auto-saving settings:", error);
        }
      };
      
      const timeoutId = setTimeout(autoSave, 500);
      return () => clearTimeout(timeoutId);
    }
  }, [selectedFormat, selectedQuality]);

  useEffect(() => {
    document.addEventListener('contextmenu', (e) => {
      e.preventDefault();
    });
  }, []);

  return (
    <div className="w-full mx-auto h-full bg-background text-foreground min-h-screen">
      <header className="border-b bg-card p-3">
        <div className="flex items-center gap-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-md">
            <svg width="32" height="32" viewBox="0 0 1024 1024" fill="none" xmlns="http://www.w3.org/2000/svg">
              <g filter="url(#filter0_i_1_19)">
                <rect x="87" y="87" width="850" height="850" rx="256" fill="#181818"/>
              </g>
              <g filter="url(#filter1_ii_1_19)">
                <path d="M712.904 536.825C732.104 525.738 732.104 498.029 712.904 486.942L553.949 395.172L394.994 303.402C375.795 292.316 351.795 306.173 351.795 328.341V511.886V695.431C351.795 717.599 375.795 731.456 394.994 720.37L553.949 628.599L712.904 536.825Z" fill="#306C54"/>
              </g>
              <defs>
                <filter id="filter0_i_1_19" x="87" y="87" width="850" height="900" filterUnits="userSpaceOnUse" colorInterpolationFilters="sRGB">
                  <feFlood floodOpacity="0" result="BackgroundImageFix"/>
                  <feBlend mode="normal" in="SourceGraphic" in2="BackgroundImageFix" result="shape"/>
                  <feColorMatrix in="SourceAlpha" type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0" result="hardAlpha"/>
                  <feMorphology radius="20" operator="erode" in="SourceAlpha" result="effect1_innerShadow_1_19"/>
                  <feOffset dy="50"/>
                  <feGaussianBlur stdDeviation="50"/>
                  <feComposite in2="hardAlpha" operator="arithmetic" k2="-1" k3="1"/>
                  <feColorMatrix type="matrix" values="0 0 0 0 0.196804 0 0 0 0 0.433519 0 0 0 0 0.336901 0 0 0 0.3 0"/>
                  <feBlend mode="normal" in2="shape" result="effect1_innerShadow_1_19"/>
                </filter>
                <filter id="filter1_ii_1_19" x="351.795" y="279.5" width="395.509" height="484.772" filterUnits="userSpaceOnUse" colorInterpolationFilters="sRGB">
                  <feFlood floodOpacity="0" result="BackgroundImageFix"/>
                  <feBlend mode="normal" in="SourceGraphic" in2="BackgroundImageFix" result="shape"/>
                  <feColorMatrix in="SourceAlpha" type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0" result="hardAlpha"/>
                  <feOffset dy="-20"/>
                  <feGaussianBlur stdDeviation="25"/>
                  <feComposite in2="hardAlpha" operator="arithmetic" k2="-1" k3="1"/>
                  <feColorMatrix type="matrix" values="0 0 0 0 1 0 0 0 0 1 0 0 0 0 1 0 0 0 0.2 0"/>
                  <feBlend mode="normal" in2="shape" result="effect1_innerShadow_1_19"/>
                  <feColorMatrix in="SourceAlpha" type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0" result="hardAlpha"/>
                  <feOffset dx="20" dy="40"/>
                  <feGaussianBlur stdDeviation="50"/>
                  <feComposite in2="hardAlpha" operator="arithmetic" k2="-1" k3="1"/>
                  <feColorMatrix type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0.8 0"/>
                  <feBlend mode="normal" in2="effect1_innerShadow_1_19" result="effect2_innerShadow_1_19"/>
                </filter>
              </defs>
            </svg>
          </div>
          <div className="flex-1">
            <h1 className="text-sm font-semibold">Snapper</h1>
            <p className="text-xs text-muted-foreground">Quick downloads</p>
          </div>
          <Badge variant="secondary" className="text-xs px-2 py-0.5">
            v1.0.0
          </Badge>
        </div>
      </header>

      <main className="p-3">
        <Tabs defaultValue="download" className="w-full">
          <TabsList className="grid w-full grid-cols-3 mb-3">
            <TabsTrigger value="download" className="text-xs px-2 py-1">
              <Download className="h-3 w-3 mr-1" />
              Download
            </TabsTrigger>
            <TabsTrigger value="history" className="text-xs px-2 py-1">
              <History className="h-3 w-3 mr-1" />
              History
            </TabsTrigger>
            <TabsTrigger value="settings" className="text-xs px-2 py-1">
              <Settings className="h-3 w-3 mr-1" />
              Settings
            </TabsTrigger>
          </TabsList>
          
          <TabsContent value="download" className="space-y-3 mt-0">
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm flex items-center">
                  Download Video
                </CardTitle>
                <CardDescription className="text-xs">
                  Paste YouTube URL and download media
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="space-y-2">
                  <Input
                    placeholder="https://www.youtube.com/watch?v=..."
                    value={url}
                    onChange={(e) => setUrl(e.target.value)}
                    className="text-sm"
                  />
                </div>
                
                <Button 
                  onClick={handleDownload}
                  disabled={isDownloading || !url.trim() || isClearingHistory}
                  className="w-full text-sm"
                >
                  {isDownloading ? "Downloading..." : "Download"}
                </Button>
                
                {isDownloading && (
                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-xs">
                      <span>Download Progress</span>
                      <span>{downloadProgress}%</span>
                    </div>
                    <Progress value={downloadProgress} className="h-2" />
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>
          
          <TabsContent value="history" className="space-y-3 mt-0">
            <Card>
              <CardHeader className="pb-1.5">
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle className="text-sm mb-1.5">Download History</CardTitle>
                    <CardDescription className="text-xs">
                      Recently downloaded videos
                    </CardDescription>
                  </div>
                  {downloadHistory.length > 0 && (
                    <Button 
                      size="sm" 
                      onClick={clearHistory}
                      disabled={isClearingHistory}
                      variant="outline"
                      className="text-xs px-2 py-1"
                    >
                      <Trash2 className="h-3 w-3 mr-1" />
                      {isClearingHistory ? "Clearing..." : "Clear"}
                    </Button>
                  )}
                </div>
              </CardHeader>
              <CardContent className="pt-0">
                <ScrollArea className="h-64">
                {downloadHistory.length === 0 ? (
                  <div className="text-center py-6">
                    <History className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                    <p className="text-xs text-muted-foreground">No download history</p>
                    <p className="text-xs text-muted-foreground mt-1">Downloaded files will appear here</p>
                  </div>
                ) : (
                  <div className="space-y-2">
                    {downloadHistory.map((item, index) => (
                      <div key={item.id}>
                        <div className="flex items-center gap-2 p-2 rounded-md border">
                          {item.status === "completed" ? (
                            <CheckCircle className="h-4 w-4 text-green-500 flex-shrink-0" />
                          ) : (
                            <AlertCircle className="h-4 w-4 text-red-500 flex-shrink-0" />
                          )}
                          <div className="flex-1 min-w-0 w-full">
                            <p className="font-medium text-xs truncate max-w-100">{item.title}</p>
                            <div className="flex items-center gap-1">
                              <p 
                                className="text-xs text-muted-foreground truncate max-w-100 cursor-pointer hover:text-blue-500/50 hover:underline transition-colors flex items-center gap-1"
                                onClick={() => openInBrowser(item.url)}
                                title="Click to open in browser"
                              >
                                {item.url}
                                <ExternalLink className="h-3 w-3 flex-shrink-0" />
                              </p>
                            </div>
                          </div>
                          <div className="text-right flex-shrink-0">
                            <Badge variant={item.status === "completed" ? "default" : "destructive"} className="text-xs">
                              {item.status === "completed" ? "OK" : "Error"}
                            </Badge>
                            <p className="text-xs text-muted-foreground mt-1">{item.date}</p>
                          </div>
                        </div>
                        {index < downloadHistory.length - 1 && <Separator className="my-2" />}
                      </div>
                    ))}
                  </div>
                )}
                </ScrollArea>
              </CardContent>
            </Card>
          </TabsContent>
          
          <TabsContent value="settings" className="space-y-3 mt-0">
            <Card>
              <CardHeader className="pb-1.5">
                <CardTitle className="text-sm">Download Settings</CardTitle>
                <CardDescription className="text-xs">
                  Customize your download preferences
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="space-y-2">
                  <label className="text-xs font-medium">Output Directory</label>
                  <div className="flex gap-1">
                    <Input 
                      placeholder="Path to downloads folder" 
                      value={outputPath}
                      onChange={(e) => setOutputPath(e.target.value)}
                      readOnly
                      className="flex-1 text-xs"
                    />
                    <Button variant="outline" onClick={selectDownloadFolder} className="px-2">
                      <FolderOpen className="h-3 w-3" />
                    </Button>
                    <Button variant="ghost" onClick={loadDefaultPath} size="sm" className="text-xs px-2">
                      Reset
                    </Button>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Click folder icon to browse
                  </p>
                </div>
              
                <div className="space-y-2">
                  <label className="text-xs font-medium">Default Format</label>
                  <Select value={selectedFormat} onValueChange={setSelectedFormat}>
                    <SelectTrigger className="text-xs">
                      <SelectValue placeholder="Select format" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="mp3">MP3 (Audio)</SelectItem>
                      <SelectItem value="wav">WAV (High Quality Audio)</SelectItem>
                      <SelectItem value="mp4">MP4 (Video)</SelectItem>
                      <SelectItem value="webm">WEBM</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <label className="text-xs font-medium">Quality</label>
                  <Select value={selectedQuality} onValueChange={setSelectedQuality}>
                    <SelectTrigger className="text-xs">
                      <SelectValue placeholder="Select quality" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="best">Best Available</SelectItem>
                      <SelectItem value="high">High</SelectItem>
                      <SelectItem value="medium">Medium</SelectItem>
                      <SelectItem value="low">Low</SelectItem>
                      <SelectItem value="worst">Worst</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
}

export default App;