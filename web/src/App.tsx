import { useState, useEffect } from 'react'
import axios from 'axios'
import { Settings, Folder, Save, Code, Play, Plus, Trash } from 'lucide-react'
import Editor from '@monaco-editor/react'

interface SettingsType {
  user: { name: string }
  runtime: { pack: { current: string; mood: string } }
}

function App() {
  const [settings, setSettings] = useState<SettingsType | null>(null)
  const [packs, setPacks] = useState<string[]>([])
  const [selectedPack, setSelectedPack] = useState<string | null>(null)
  const [packConfig, setPackConfig] = useState<any>(null)
  const [activeTab, setActiveTab] = useState<'settings' | 'packs' | 'code'>('settings')
  const [code, setCode] = useState<string>('// Write your code here\ngoon.system.log("Hello World!");\n')
  const [logs, setLogs] = useState<string[]>([])
  const [isRunning, setIsRunning] = useState(false)

  useEffect(() => {
    fetchSettings()
    fetchPacks()
  }, [])

  const handleEditorDidMount = async (_editor: any, monaco: any) => {
    try {
      const res = await axios.get('/api/sdk')
      const libSource = res.data

      monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
        target: monaco.languages.typescript.ScriptTarget.ES2020,
        allowNonTsExtensions: true,
        moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
        module: monaco.languages.typescript.ModuleKind.CommonJS,
        noEmit: true,
        typeRoots: ["node_modules/@types"]
      });

      monaco.languages.typescript.typescriptDefaults.addExtraLib(
        libSource,
        'ts:filename/goon-sdk.d.ts'
      );
    } catch (e) {
      console.error("Failed to load SDK definitions", e)
    }
  }

  const fetchSettings = async () => {
    const res = await axios.get('/api/settings')
    setSettings(res.data)
  }

  const fetchPacks = async () => {
    const res = await axios.get('/api/packs')
    setPacks(res.data)
  }

  const saveSettings = async () => {
    if (!settings) return
    await axios.post('/api/settings', settings)
    alert('Settings saved!')
  }

  const loadPack = async (name: string) => {
    const res = await axios.get(`/api/packs/${name}`)
    setPackConfig(res.data)
    setSelectedPack(name)
  }

  const savePack = async () => {
    if (!selectedPack || !packConfig) return

    // Clean up tags before saving
    const cleanConfig = {
      ...packConfig,
      assets: {
        ...packConfig.assets,
        image: packConfig.assets.image?.map((img: any) => ({
          ...img,
          tags: img.tags.map((t: string) => t.trim()).filter(Boolean)
        })),
        wallpaper: packConfig.assets.wallpaper?.map((img: any) => ({
          ...img,
          tags: img.tags.map((t: string) => t.trim()).filter(Boolean)
        }))
      },
      moods: packConfig.moods?.map((mood: any) => ({
        ...mood,
        tags: mood.tags.map((t: string) => t.trim()).filter(Boolean)
      })),
      websites: packConfig.websites?.map((site: any) => ({
        ...site,
        tags: site.tags.map((t: string) => t.trim()).filter(Boolean)
      })),
      prompts: packConfig.prompts
    }

    await axios.post(`/api/packs/${selectedPack}`, cleanConfig)
    alert('Pack saved!')
  }

  const handleUpload = async (e: React.ChangeEvent<HTMLInputElement>, type: string) => {
    if (!e.target.files || !selectedPack) return

    const formData = new FormData()
    for (let i = 0; i < e.target.files.length; i++) {
      formData.append('files', e.target.files[i])
    }

    try {
      await axios.post(`/api/packs/${selectedPack}/assets/${type}`, formData, {
        headers: { 'Content-Type': 'multipart/form-data' }
      })
      alert('Upload successful!')
      loadPack(selectedPack) // Reload to show new assets
    } catch (err) {
      alert('Upload failed')
    }
  }

  const runCode = async () => {
    setIsRunning(true)
    setLogs([])
    try {
      const res = await axios.post('/api/run', { code })
      setLogs(res.data.logs)
      if (res.data.error) {
        setLogs(prev => [...prev, `Error: ${res.data.error}`])
      }
    } catch (e: any) {
      setLogs(prev => [...prev, `Request failed: ${e.message}`])
    } finally {
      setIsRunning(false)
    }
  }

  const createPack = async () => {
    const name = prompt("Enter new pack name:")
    if (!name) return

    try {
      await axios.post('/api/packs', { name })
      alert('Pack created!')
      fetchPacks()
    } catch (e: any) {
      alert('Failed to create pack: ' + (e.response?.data || e.message))
    }
  }

  const ALL_PERMISSIONS = ['image', 'video', 'audio', 'hypno', 'wallpaper', 'prompt', 'website'];

  if (!settings) return <div className="p-8 text-white">Loading...</div>

  return (
    <div className="min-h-screen bg-gray-900 text-white flex">
      {/* Sidebar */}
      <div className="w-64 bg-gray-800 p-4 flex flex-col gap-2">
        <h1 className="text-xl font-bold mb-4 px-2">GoonAI Config</h1>

        <button
          onClick={() => setActiveTab('settings')}
          className={`flex items-center gap-2 p-2 rounded ${activeTab === 'settings' ? 'bg-blue-600' : 'hover:bg-gray-700'}`}
        >
          <Settings size={20} /> Settings
        </button>

        <button
          onClick={() => setActiveTab('code')}
          className={`flex items-center gap-2 p-2 rounded ${activeTab === 'code' ? 'bg-blue-600' : 'hover:bg-gray-700'}`}
        >
          <Code size={20} /> Code Editor
        </button>

        <div className="mt-4 px-2 text-sm text-gray-400 uppercase flex justify-between items-center">
          <span>Packs</span>
          <button onClick={createPack} className="hover:text-white p-1 rounded hover:bg-gray-700" title="Create Pack">
            <Plus size={16} />
          </button>
        </div>
        {packs.map(pack => (
          <button
            key={pack}
            onClick={() => { setActiveTab('packs'); loadPack(pack); }}
            className={`flex items-center gap-2 p-2 rounded ${activeTab === 'packs' && selectedPack === pack ? 'bg-blue-600' : 'hover:bg-gray-700'}`}
          >
            <Folder size={20} /> {pack}
          </button>
        ))}
      </div>

      {/* Main Content */}
      <div className="flex-1 p-8 overflow-y-auto">
        {activeTab === 'settings' && (
          <div className="max-w-2xl">
            <h2 className="text-2xl font-bold mb-6">Settings</h2>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">User Name</label>
                <input
                  type="text"
                  value={settings.user.name}
                  onChange={e => setSettings({...settings, user: { ...settings.user, name: e.target.value }})}
                  className="w-full bg-gray-800 border border-gray-700 rounded p-2"
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Current Pack</label>
                <select
                  value={settings.runtime.pack.current}
                  onChange={e => setSettings({...settings, runtime: { ...settings.runtime, pack: { ...settings.runtime.pack, current: e.target.value }}})}
                  className="w-full bg-gray-800 border border-gray-700 rounded p-2"
                >
                  {packs.map(p => <option key={p} value={p}>{p}</option>)}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Initial Mood</label>
                <input
                  type="text"
                  value={settings.runtime.pack.mood}
                  onChange={e => setSettings({...settings, runtime: { ...settings.runtime, pack: { ...settings.runtime.pack, mood: e.target.value }}})}
                  className="w-full bg-gray-800 border border-gray-700 rounded p-2"
                />
              </div>

              <button onClick={saveSettings} className="flex items-center gap-2 bg-green-600 px-4 py-2 rounded hover:bg-green-700">
                <Save size={20} /> Save Settings
              </button>
            </div>
          </div>
        )}

        {activeTab === 'code' && (
          <div className="h-full flex flex-col">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-2xl font-bold">Code Editor</h2>
              <button
                onClick={runCode}
                disabled={isRunning}
                className={`flex items-center gap-2 px-4 py-2 rounded ${isRunning ? 'bg-gray-600' : 'bg-green-600 hover:bg-green-700'}`}
              >
                <Play size={20} /> {isRunning ? 'Running...' : 'Run Code'}
              </button>
            </div>

            <div className="flex-1 grid grid-rows-2 gap-4 min-h-0">
              <div className="h-full border border-gray-700 rounded overflow-hidden">
                <Editor
                  height="100%"
                  defaultLanguage="typescript"
                  theme="vs-dark"
                  value={code}
                  onChange={(value) => setCode(value || '')}
                  onMount={handleEditorDidMount}
                  options={{
                    minimap: { enabled: false },
                    fontSize: 14,
                    scrollBeyondLastLine: false,
                    automaticLayout: true,
                  }}
                />
              </div>

              <div className="bg-black rounded p-4 font-mono text-sm overflow-y-auto border border-gray-800">
                <div className="text-gray-500 mb-2">Output:</div>
                {logs.map((log, i) => (
                  <div key={i} className="text-green-400 border-b border-gray-900 py-1">{log}</div>
                ))}
                {logs.length === 0 && <div className="text-gray-600 italic">No output</div>}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'packs' && packConfig && (
          <div>
            <div className="flex justify-between items-center mb-6">
              <h2 className="text-2xl font-bold">Pack: {selectedPack}</h2>
              <button onClick={savePack} className="flex items-center gap-2 bg-green-600 px-4 py-2 rounded hover:bg-green-700">
                <Save size={20} /> Save Pack
              </button>
            </div>

            <div className="space-y-8">
              {/* Metadata */}
              <div className="bg-gray-800 p-4 rounded">
                <h3 className="text-lg font-semibold mb-4">Metadata</h3>
                <div className="grid grid-cols-2 gap-4 mb-4">
                  <div>
                    <label className="block text-sm font-medium mb-1">Name</label>
                    <input
                      value={packConfig.meta.name}
                      onChange={e => setPackConfig({...packConfig, meta: {...packConfig.meta, name: e.target.value}})}
                      className="w-full bg-gray-700 border border-gray-600 rounded p-2"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-1">Version</label>
                    <input
                      value={packConfig.meta.version}
                      onChange={e => setPackConfig({...packConfig, meta: {...packConfig.meta, version: e.target.value}})}
                      className="w-full bg-gray-700 border border-gray-600 rounded p-2"
                    />
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium mb-2">Permissions</label>
                  <div className="flex flex-wrap gap-2">
                    {ALL_PERMISSIONS.map(perm => (
                      <label key={perm} className="flex items-center gap-2 bg-gray-700 px-3 py-1 rounded cursor-pointer hover:bg-gray-600">
                        <input
                          type="checkbox"
                          checked={packConfig.meta.permissions?.includes(perm)}
                          onChange={() => {
                            const current = packConfig.meta.permissions || [];
                            const newPerms = current.includes(perm)
                              ? current.filter((p: string) => p !== perm)
                              : [...current, perm];
                            setPackConfig({...packConfig, meta: {...packConfig.meta, permissions: newPerms}});
                          }}
                        />
                        <span className="capitalize">{perm}</span>
                      </label>
                    ))}
                  </div>
                </div>
              </div>

              {/* System Prompt */}
              <div className="bg-gray-800 p-4 rounded">
                <h3 className="text-lg font-semibold mb-4">System Prompt</h3>
                <textarea
                  value={packConfig.prompts?.system || ''}
                  onChange={e => setPackConfig({
                    ...packConfig,
                    prompts: { ...packConfig.prompts, system: e.target.value }
                  })}
                  className="w-full bg-gray-700 border border-gray-600 rounded p-2 h-32 font-mono text-sm"
                  placeholder="Enter system prompt override..."
                />
              </div>

              {/* Moods */}
              <div className="bg-gray-800 p-4 rounded">
                <div className="flex justify-between items-center mb-4">
                  <h3 className="text-lg font-semibold">Moods</h3>
                  <button
                    onClick={() => setPackConfig({
                      ...packConfig,
                      moods: [...(packConfig.moods || []), { name: 'new_mood', description: '', tags: [] }]
                    })}
                    className="flex items-center gap-1 bg-blue-600 px-2 py-1 rounded text-sm hover:bg-blue-700"
                  >
                    <Plus size={16} /> Add Mood
                  </button>
                </div>
                <div className="space-y-4">
                  {packConfig.moods?.map((mood: any, idx: number) => (
                    <div key={idx} className="bg-gray-700 p-3 rounded relative group">
                      <button
                        onClick={() => {
                          const newMoods = [...packConfig.moods];
                          newMoods.splice(idx, 1);
                          setPackConfig({...packConfig, moods: newMoods});
                        }}
                        className="absolute top-2 right-2 text-gray-400 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity"
                      >
                        <Trash size={16} />
                      </button>
                      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div>
                          <label className="block text-xs text-gray-400 mb-1">Name</label>
                          <input
                            value={mood.name}
                            onChange={e => {
                              const newMoods = [...packConfig.moods];
                              newMoods[idx].name = e.target.value;
                              setPackConfig({...packConfig, moods: newMoods});
                            }}
                            className="w-full bg-gray-600 border border-gray-500 rounded p-1 text-sm"
                          />
                        </div>
                        <div>
                          <label className="block text-xs text-gray-400 mb-1">Description</label>
                          <input
                            value={mood.description}
                            onChange={e => {
                              const newMoods = [...packConfig.moods];
                              newMoods[idx].description = e.target.value;
                              setPackConfig({...packConfig, moods: newMoods});
                            }}
                            className="w-full bg-gray-600 border border-gray-500 rounded p-1 text-sm"
                          />
                        </div>
                        <div>
                          <label className="block text-xs text-gray-400 mb-1">Tags (comma separated)</label>
                          <input
                            value={mood.tags.join(', ')}
                            onChange={e => {
                              const newMoods = [...packConfig.moods];
                              newMoods[idx].tags = e.target.value.split(',').map((s: string) => s.trimStart());
                              setPackConfig({...packConfig, moods: newMoods});
                            }}
                            className="w-full bg-gray-600 border border-gray-500 rounded p-1 text-sm"
                          />
                        </div>
                        <div className="col-span-1 md:col-span-3">
                          <label className="block text-xs text-gray-400 mb-1">Prompt</label>
                          <textarea
                            value={mood.prompt || ''}
                            onChange={e => {
                              const newMoods = [...packConfig.moods];
                              newMoods[idx].prompt = e.target.value;
                              setPackConfig({...packConfig, moods: newMoods});
                            }}
                            className="w-full bg-gray-600 border border-gray-500 rounded p-1 text-sm h-20 font-mono"
                            placeholder="Mood specific instructions..."
                          />
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Websites */}
              <div className="bg-gray-800 p-4 rounded">
                <div className="flex justify-between items-center mb-4">
                  <h3 className="text-lg font-semibold">Websites</h3>
                  <button
                    onClick={() => setPackConfig({
                      ...packConfig,
                      websites: [...(packConfig.websites || []), { name: 'New Site', url: 'https://', description: '', tags: [] }]
                    })}
                    className="flex items-center gap-1 bg-blue-600 px-2 py-1 rounded text-sm hover:bg-blue-700"
                  >
                    <Plus size={16} /> Add Website
                  </button>
                </div>
                <div className="space-y-4">
                  {packConfig.websites?.map((site: any, idx: number) => (
                    <div key={idx} className="bg-gray-700 p-3 rounded relative group">
                      <button
                        onClick={() => {
                          const newSites = [...packConfig.websites];
                          newSites.splice(idx, 1);
                          setPackConfig({...packConfig, websites: newSites});
                        }}
                        className="absolute top-2 right-2 text-gray-400 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity"
                      >
                        <Trash size={16} />
                      </button>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-2">
                        <div>
                          <label className="block text-xs text-gray-400 mb-1">Name</label>
                          <input
                            value={site.name}
                            onChange={e => {
                              const newSites = [...packConfig.websites];
                              newSites[idx].name = e.target.value;
                              setPackConfig({...packConfig, websites: newSites});
                            }}
                            className="w-full bg-gray-600 border border-gray-500 rounded p-1 text-sm"
                          />
                        </div>
                        <div>
                          <label className="block text-xs text-gray-400 mb-1">URL</label>
                          <input
                            value={site.url}
                            onChange={e => {
                              const newSites = [...packConfig.websites];
                              newSites[idx].url = e.target.value;
                              setPackConfig({...packConfig, websites: newSites});
                            }}
                            className="w-full bg-gray-600 border border-gray-500 rounded p-1 text-sm"
                          />
                        </div>
                      </div>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                          <label className="block text-xs text-gray-400 mb-1">Description</label>
                          <input
                            value={site.description}
                            onChange={e => {
                              const newSites = [...packConfig.websites];
                              newSites[idx].description = e.target.value;
                              setPackConfig({...packConfig, websites: newSites});
                            }}
                            className="w-full bg-gray-600 border border-gray-500 rounded p-1 text-sm"
                          />
                        </div>
                        <div>
                          <label className="block text-xs text-gray-400 mb-1">Tags</label>
                          <input
                            value={site.tags.join(', ')}
                            onChange={e => {
                              const newSites = [...packConfig.websites];
                              newSites[idx].tags = e.target.value.split(',').map((s: string) => s.trimStart());
                              setPackConfig({...packConfig, websites: newSites});
                            }}
                            className="w-full bg-gray-600 border border-gray-500 rounded p-1 text-sm"
                          />
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Upload */}
              <div className="bg-gray-800 p-4 rounded">
                <h3 className="text-lg font-semibold mb-4">Upload Assets</h3>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div>
                    <label className="block text-sm font-medium mb-1">Images</label>
                    <input type="file" multiple accept="image/*" onChange={(e) => handleUpload(e, 'image')} className="text-sm text-gray-400 w-full" />
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-1">Videos</label>
                    <input type="file" multiple accept="video/*" onChange={(e) => handleUpload(e, 'video')} className="text-sm text-gray-400 w-full" />
                  </div>
                   <div>
                    <label className="block text-sm font-medium mb-1">Audio</label>
                    <input type="file" multiple accept="audio/*" onChange={(e) => handleUpload(e, 'audio')} className="text-sm text-gray-400 w-full" />
                  </div>
                   <div>
                    <label className="block text-sm font-medium mb-1">Wallpapers</label>
                    <input type="file" multiple accept="image/*" onChange={(e) => handleUpload(e, 'wallpaper')} className="text-sm text-gray-400 w-full" />
                  </div>
                </div>
              </div>

              {/* Assets */}
              <div className="bg-gray-800 p-4 rounded">
                <h3 className="text-lg font-semibold mb-4">Images</h3>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  {packConfig.assets.image?.map((img: any, idx: number) => (
                    <div key={idx} className="bg-gray-700 p-2 rounded">
                      <div className="aspect-video bg-black mb-2 rounded overflow-hidden">
                        <img
                          src={`/packs/${selectedPack}/${img.path}`}
                          alt={img.path}
                          className="w-full h-full object-contain"
                        />
                      </div>
                      <div className="text-xs text-gray-400 mb-1 truncate">{img.path}</div>
                      <input
                        placeholder="Tags (comma separated)"
                        value={img.tags.join(', ')}
                        onChange={e => {
                          const newImages = [...packConfig.assets.image];
                          newImages[idx].tags = e.target.value.split(',').map((s: string) => s.trimStart());
                          setPackConfig({...packConfig, assets: {...packConfig.assets, image: newImages}});
                        }}
                        className="w-full bg-gray-600 border border-gray-500 rounded p-1 text-sm"
                      />
                    </div>
                  ))}
                </div>
              </div>

              <div className="bg-gray-800 p-4 rounded">
                <h3 className="text-lg font-semibold mb-4">Wallpapers</h3>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  {packConfig.assets.wallpaper?.map((img: any, idx: number) => (
                    <div key={idx} className="bg-gray-700 p-2 rounded">
                      <div className="aspect-video bg-black mb-2 rounded overflow-hidden">
                        <img
                          src={`/packs/${selectedPack}/${img.path}`}
                          alt={img.path}
                          className="w-full h-full object-cover"
                        />
                      </div>
                      <div className="text-xs text-gray-400 mb-1 truncate">{img.path}</div>
                      <input
                        placeholder="Tags (comma separated)"
                        value={img.tags.join(', ')}
                        onChange={e => {
                          const newWallpapers = [...(packConfig.assets.wallpaper || [])];
                          newWallpapers[idx].tags = e.target.value.split(',').map((s: string) => s.trimStart());
                          setPackConfig({...packConfig, assets: {...packConfig.assets, wallpaper: newWallpapers}});
                        }}
                        className="w-full bg-gray-600 border border-gray-500 rounded p-1 text-sm"
                      />
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default App
