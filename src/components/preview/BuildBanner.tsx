import React from 'react';
import { CheckCircle2, AlertCircle, UploadCloud } from 'lucide-react';
import type { BuildResult } from '../../types';

interface BuildBannerProps {
  buildResult: BuildResult | null;
  publishMessage: string | null;
}

export function BuildBanner({ buildResult, publishMessage }: BuildBannerProps) {
  if (!buildResult && !publishMessage) return null;

  return (
    <>
      {buildResult && (
        <div
          className={`px-6 py-2 text-xs border-b flex items-center justify-between ${
            buildResult.success
              ? 'bg-emerald-950/40 border-emerald-800 text-emerald-300'
              : 'bg-red-950/40 border-red-800 text-red-300'
          }`}
        >
          <div className="flex items-center gap-2">
            {buildResult.success ? (
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            ) : (
              <AlertCircle className="w-4 h-4 text-red-400" />
            )}
            <span>
              {buildResult.success
                ? `Build succeeded: ${buildResult.pages_built} pages built, ${buildResult.images_generated} images optimized.`
                : `Build failed with ${buildResult.errors.length} error(s).`}
            </span>
          </div>
          {buildResult.errors.length > 0 && (
            <div className="text-right">
              {buildResult.errors[0].file}:{buildResult.errors[0].line} —{' '}
              {buildResult.errors[0].message}
            </div>
          )}
        </div>
      )}

      {publishMessage && (
        <div className="px-6 py-2 bg-indigo-950/40 border-b border-indigo-800 text-indigo-300 text-xs flex items-center gap-2">
          <UploadCloud className="w-3.5 h-3.5 text-indigo-400" />
          <span>{publishMessage}</span>
        </div>
      )}
    </>
  );
}
