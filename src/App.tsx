function App() {
  return (
    <div className="min-h-screen bg-gray-900 text-white flex flex-col items-center justify-center p-8">
      <h1 className="text-4xl font-bold mb-4">VTuber Overlay Suite</h1>
      <p className="text-gray-400 mb-8">VTuber streaming support tool</p>
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-gray-800 rounded-lg p-6 hover:bg-gray-700 transition-colors cursor-pointer">
          <h2 className="text-xl font-semibold mb-2">Comment Overlay</h2>
          <p className="text-gray-400 text-sm">Display live chat comments on stream</p>
        </div>
        <div className="bg-gray-800 rounded-lg p-6 hover:bg-gray-700 transition-colors cursor-pointer">
          <h2 className="text-xl font-semibold mb-2">Setlist Manager</h2>
          <p className="text-gray-400 text-sm">Manage and display song setlists</p>
        </div>
      </div>
      <p className="text-gray-500 text-sm mt-8">Version 0.1.0</p>
    </div>
  )
}

export default App
