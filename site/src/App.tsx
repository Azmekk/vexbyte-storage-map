import { useState } from "react";
import { Closing } from "./components/Closing";
import { FeatureGrid } from "./components/FeatureGrid";
import { Footer } from "./components/Footer";
import { Hero } from "./components/Hero";
import { Nav } from "./components/Nav";
import { Showcase } from "./components/Showcase";
import { ViewTabs } from "./components/ViewTabs";
import { useDownloadTarget } from "./hooks/useDownloadTarget";
import type { StartedDownload } from "./lib/release/startedDownload";

export function App() {
  const downloadTarget = useDownloadTarget();
  const [startedDownload, setStartedDownload] = useState<StartedDownload | null>(null);

  return (
    <>
      <Nav />
      <main id="top">
        <Hero downloadTarget={downloadTarget} onDownloadStarted={setStartedDownload} />
        <Showcase release={downloadTarget.release} />
        <ViewTabs />
        <FeatureGrid />
        <Closing
          downloadTarget={downloadTarget}
          onDownloadStarted={setStartedDownload}
          startedDownload={startedDownload}
        />
      </main>
      <Footer />
    </>
  );
}
