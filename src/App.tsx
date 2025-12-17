import { ApiKeySetup } from './components/ApiKeySetup';

function App() {
  return (
    <div className="min-h-screen bg-gray-50">
      <div className="container mx-auto py-8">
        <h1 className="text-4xl font-bold mb-2 text-gray-900">
          VTuber Overlay Suite
        </h1>
        <p className="text-gray-600 mb-8">VTuber streaming support tool</p>
        <ApiKeySetup />
      </div>
    </div>
  );
}

export default App;
