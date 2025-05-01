import { memo } from "react";
import { Card, CardContent } from "./card";

export interface DetailEmptyViewProps {
  message: string;
}

export const DetailEmptyView = memo(({ message }: DetailEmptyViewProps) => {
  return (
    <div className="container mx-auto py-6 max-w-4xl">
      <Card>
        <CardContent className="flex items-center justify-center h-32">
          <p className="text-muted-foreground">{message ?? "No data"}</p>
        </CardContent>
      </Card>
    </div>
  );
});
