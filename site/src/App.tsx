import { useState } from "react";
import { Closing } from "./components/Closing";
import { FeatureGrid } from "./components/FeatureGrid";
import { Footer } from "./components/Footer";
import { Hero } from "./components/Hero";
import { Nav } from "./components/Nav";
import { Showcase } from "./components/Showcase";
import { ViewTabs } from "./components/ViewTabs";
import { useDownloadTarget } from "./hooks/useDownloadTarget";

export function App() {
  const downloadTarget = useDownloadTarget();
  const [hasStartedDownload, setHasStartedDownload] = useState(false);
  const onDownloadStarted = () => setHasStartedDownload(true);

  return (
    <>
      <Nav />
      <main id="top">
        <Hero downloadTarget={downloadTarget} onDownloadStarted={onDownloadStarted} />
        <Showcase release={downloadTarget.release} />
        <ViewTabs />
        <FeatureGrid />
        <Closing
          downloadTarget={downloadTarget}
          onDownloadStarted={onDownloadStarted}
          showSmartScreenNote={hasStartedDownload}
        />
      </main>
      <Footer />
    </>
  );
}
